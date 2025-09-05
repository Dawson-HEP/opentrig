//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.
#![allow(warnings)] 
#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use crate::{dac::DacError, fpga::{daq_fpga_clock_config, daq_fpga_spi_config, DAQFpga}};
use cortex_m::peripheral::nvic;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{gpio::Pin, pac::pio::Irq, pio::IrqFlags, pwm::Pwm, spi::Spi, usb::Bus};
//use embassy_rp::{gpio::Pin, pac::pio::Irq, pio::{Irq, IrqFlags}, pwm::Pwm, spi::Spi};

use {defmt_rtt as _, panic_probe as _};

use embassy_rp::gpio::{Level, Output};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_sync::channel::Channel;
use embassy_time::Timer;

use defmt::info;
use embassy_executor::Executor;
use embassy_rp::interrupt;
use embassy_rp::uart::{self, UartTx};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex;
use static_cell::StaticCell;




use embassy_futures::join::join;

use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
//use embassy_rp::usb::InterruptHandler;
use embassy_rp::usb::{Driver, InterruptHandler};

//use embassy_usb::driver::{Driver, Endpoint, EndpointIn, EndpointOut};
use embassy_usb::driver::{Endpoint, EndpointIn, EndpointOut};
use embassy_usb::msos::{self, windows_version};
use embassy_usb::{Builder, Config};

use core::str;




mod data;
mod fpga;
mod dac;
mod handle_inputs;

use dac::DacManager;
use data::DAQSample;


use core::{fmt, ops::Deref, u64};
use serde::{Serialize, Deserialize};
use postcard::{from_bytes, to_vec, to_vec_cobs};
use heapless::Vec;



static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
static DAQ_CHANNEL: Channel<CriticalSectionRawMutex, DAQSample, 1> = Channel::new();
//static INPUT_CHANNEL: Channel<CriticalSectionRawMutex, &[u8], 64> = Channel::new();
static INPUT_CHANNEL: Channel<CriticalSectionRawMutex, [u8;64], 1> = Channel::new();


// static buffers for embassy-usb for ownership reasons
//     Create embassy-usb DeviceBuilder using the driver and config.
//     It needs some buffers for building the descriptors.
static mut config_descriptor : [u8;256] = [0; 256];
static mut bos_descriptor : [u8;256]  = [0; 256];
static mut msos_descriptor : [u8;256]  = [0; 256];
static mut control_buf : [u8;64]  = [0; 64];



// This is a randomly generated GUID to allow clients on Windows to find our device
const DEVICE_INTERFACE_GUIDS: &[&str] = &["{AFB9A6FB-30BA-44BC-9232-806CFC875321}"];

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});




#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_rp::Peripherals = embassy_rp::init(Default::default());


    let ldacs = [
        Output::new(p.PIN_4, Level::Low),
        Output::new(p.PIN_5, Level::Low),
        Output::new(p.PIN_6, Level::Low),
        Output::new(p.PIN_7, Level::Low),
        Output::new(p.PIN_8, Level::Low),
        Output::new(p.PIN_9, Level::Low),
    ];

    let mut dac_manager: DacManager<'_> = DacManager::new(p.I2C0, p.PIN_1, p.PIN_0, ldacs);

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| unwrap!(spawner.spawn(core1_task(p.USB))));
        },
    );
    
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| unwrap!(spawner.spawn(core0_task(dac_manager))));

}


async fn make_usb(usb_pin:USB) ->
    (embassy_usb::UsbDevice<'static, Driver<'static, USB>>,
    embassy_rp::usb::Endpoint<'static, USB, embassy_rp::usb::Out>,
    embassy_rp::usb::Endpoint<'static, USB, embassy_rp::usb::In>,) {
    let driver = Driver::new(usb_pin, Irqs);

    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB raw example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    
    
    // call static buffers from earlier
    let mut builder: Builder<'_, Driver<'_, USB>> = unsafe { Builder::new(
            driver,
            config,
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut msos_descriptor,
            &mut control_buf,
        )
    };

    // Add the Microsoft OS Descriptor (MSOS/MOD) descriptor.
    // We tell Windows that this entire device is compatible with the "WINUSB" feature,
    // which causes it to use the built-in WinUSB driver automatically, which in turn
    // can be used by libusb/rusb software without needing a custom driver or INF file.
    // In principle you might want to call msos_feature() just on a specific function,
    // if your device also has other functions that still use standard class drivers.
    builder.msos_descriptor(windows_version::WIN8_1, 0);
    builder.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
    builder.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
        "DeviceInterfaceGUIDs",
        msos::PropertyData::RegMultiSz(DEVICE_INTERFACE_GUIDS),
    ));

    // Add a vendor-specific function (class 0xFF), and corresponding interface,
    // that uses our custom handler.
    let mut function = builder.function(0xFF, 0, 0);
    let mut interface = function.interface();
    let mut alt = interface.alt_setting(0xFF, 0, 0, None);
    let mut read_ep : embassy_rp::usb::Endpoint<'static, USB, embassy_rp::usb::Out> = alt.endpoint_bulk_out(64);
    let mut write_ep: embassy_rp::usb::Endpoint<'static, USB, embassy_rp::usb::In> = alt.endpoint_bulk_in(64);
    drop(function);

    // Build the builder.
    let mut usb: embassy_usb::UsbDevice<'_, Driver<'_, USB>> = builder.build();
    (usb, read_ep, write_ep)
}



#[embassy_executor::task]
async fn core0_task(dac_manager:DacManager<'static>) {
    info!("Hello from core 0");
    loop {
        let daq = DAQSample {
            trigger_id: 1,
            trigger_clk: u64::MAX,
            trigger_data: 3,
            veto_in: true,
            internal_trigger: false,
        };
        DAQ_CHANNEL.send(daq).await;
        Timer::after_millis(500).await;
    }
}

async fn use_usb(
    mut read_ep:&mut embassy_rp::usb::Endpoint<'static, USB, embassy_rp::usb::Out>,
    mut write_ep:&mut embassy_rp::usb::Endpoint<'static, USB, embassy_rp::usb::In>
    ) {
        let daqs = DAQ_CHANNEL.receive().await;
        read_ep.wait_enabled().await;
        info!("Connected");
        loop {
            let mut data = [0; 64];
            let nbytes = read_ep.read(&mut data).await.expect("failed to read endpoint");
            match data[0] {
                0xFF => {
                    //INPUT_CHANNEL.send(&data[1..nbytes]).await;
                    INPUT_CHANNEL.send(data).await;
                    match_cli_values_to_functions(&data[1..nbytes]);
                    println!("{:?}", data[1..nbytes]);
                    let output = to_vec::<DAQSample, {64 as usize}>(&daqs).unwrap();
                    write_ep.write(&output).await.ok();

                },
                _ => Err("invalid starting byte").unwrap(),
            }
        }
        info!("Disconnected");
    }


#[embassy_executor::task]
async fn core1_task(usb_pin:USB) {
    let (mut usb, mut read_ep, mut write_ep) = make_usb(usb_pin).await;

    info!("Hello from core 1");

    let usb_fut = usb.run();
    let user = use_usb(&mut read_ep, &mut write_ep);

    join(usb_fut, user).await;


    // DOES NOTHING at the moment, just for reference (as it does work, if the join() above is removed)
    loop {
        let daqs = DAQ_CHANNEL.receive().await;
        let output = to_vec::<DAQSample, {64 as usize}>(&daqs).unwrap();
        let restruct = from_bytes::<DAQSample>(output.deref()).unwrap();
        //println!("core 1 received: {:?}", re);
        //println!("core 1 received from core 0: {:?}", re);
    }
}


/// functions and their inputs
/// NOTE : inputs ending in 0 (ie 0, 10, 20, etc) are reserved for errors
/// fn                       |  input_0(function_id) |  input_1  |  input_2  |  input_3     |  input_4
/// set_voltage              |         1             |   dac_id  |  channel  | voltage u8_1 | voltage u8_2
/// set_vref_mode            |         2             |   dac_id  |  channel  |    mode      |    N/A
/// set_gain_mode            |         3             |   dac_id  |  channel  |    mode      |    N/A
/// set_power_down_mode      |         4             |   dac_id  |  channel  |    mode      |    N/A
/// set_all_voltages         |         5             |  voltages |    N/A    |    N/A       |    N/A
/// set_all_vref_modes       |         6             |    modes  |    N/A    |    N/A       |    N/A
/// set_all_gain_modes       |         7             |    modes  |    N/A    |    N/A       |    N/A
/// set_all_power_down_modes |         8             |    modes  |    N/A    |    N/A       |    N/A
/// others?


async fn match_cli_values_to_functions(inputs:&[u8]) {//, dac_manager:DacManager) {
    match inputs[0] {
        0 => {println!("error in input, function 0 is undefined")}
        1 => {} // handle_inputs::call_set_voltage(inputs, dac_manager) } // DONE
        2 => {} // handle_inputs::call_set_vref_mode(inputs, dac_manager) } // DONE
        3 => {} // handle_inputs::call_set_gain_mode(inputs, dac_manager) } // DONE
        4 => {} // handle_inputs::call_set_power_down_mode(inputs, dac_manager) } // DONE
        5 => {} // handle_inputs::call_set_all_voltages(inputs, dac_manager) }
        6 => {} // handle_inputs::call_set_all_vref_modes(inputs, dac_manager) }
        7 => {} // handle_inputs::call_set_all_gain_modes(inputs, dac_manager) }
        8 => {} // handle_inputs::call_set_all_power_down_modes(inputs, dac_manager) }
        f => {println!("error in input, function {} is undefined", f)}
    }
}