//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.
#![allow(warnings)] 
#![no_std]
#![no_main]

use crate::{dac::DacError, fpga::{daq_fpga_clock_config, daq_fpga_spi_config, DAQFpga}};
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

use dac::DacManager;
use data::DAQSample;


use core::{fmt, ops::Deref, u64};
use serde::{Serialize, Deserialize};
use postcard::{from_bytes, to_vec, to_vec_cobs};
use heapless::Vec;



static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
static CHANNEL: Channel<CriticalSectionRawMutex, &str, 1> = Channel::new();
static DAQ_CHANNEL: Channel<CriticalSectionRawMutex, DAQSample, 1> = Channel::new();




// This is a randomly generated GUID to allow clients on Windows to find our device
const DEVICE_INTERFACE_GUIDS: &[&str] = &["{AFB9A6FB-30BA-44BC-9232-806CFC875321}"];

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});


async fn send_data(n:usize, data:&[u8],
    sender : &mut embassy_rp::usb::Endpoint<'_, USB, embassy_rp::usb::In>) {
    //sender.write(&data[..n]).await.ok();
}


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());


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
            executor1.run(|spawner| unwrap!(spawner.spawn(core1_task())));
        },
    );
    
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| unwrap!(spawner.spawn(core0_task(dac_manager))));



    // Create the driver, from the HAL.
    //let driver = Driver::new(p.USB, Irqs);
    let driver = Driver::new(p.USB, Irqs);

    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB raw example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];



    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

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
    let mut read_ep = alt.endpoint_bulk_out(None, 64);
    let mut write_ep = alt.endpoint_bulk_in(None, 64);
    drop(function);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    
    // Do stuff with the class!
    let echo_fut = async {
        loop {
            read_ep.wait_enabled().await;
            info!("Connected");
            loop {
                let mut data = [0; 64];
                match read_ep.read(&mut data).await {
                    Ok(n) => {//led_blink(&mut led).await;
                        info!("Got bulk: {:a}", data[..n]);
                        // Echo back to the host:
                        send_data(n, &data, &mut write_ep).await;//, write_ep.write)
                        //write_ep.write(&data[..n]).await.ok();
                        //let a = write_ep.write(buf)
                        //for u in a {
                        //    write_ep.write(&[*u]).await.ok();
                        //}
                        //write_ep.write(&a).await.ok();
                    }
                    Err(_) => break,
                }
            }
            info!("Disconnected");
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, echo_fut).await;
    
}

//impl Format for heapless::vec::Vec<u8, 64> {
//    fn format(&self, fmt: Formatter) {
//        
//    }
//}

#[embassy_executor::task]
async fn core0_task(dac_manager:DacManager<'static>) {
    info!("Hello from core 0");
    loop {
        //CHANNEL.send("from core 0").await;
        
        let daq = DAQSample {
            trigger_id: 1,
            trigger_clk: u64::MAX,
            trigger_data: 3,
            veto_in: true,
            internal_trigger: false,
        };
        DAQ_CHANNEL.send(daq).await;
        //CHANNEL.send(LedState::On).await;
        Timer::after_millis(100).await;
        //CHANNEL.send(LedState::Off).await;
        Timer::after_millis(400).await;
        //{
        //    let mut uart = uart.lock().await;
        //    uart.write(b"core 0 sent").await.unwrap();
        //    // The uart lock is released when it goes out of scope
        //}
    }
}

#[embassy_executor::task]
async fn core1_task() {
    info!("Hello from core 1");
    loop {
        //let msg = CHANNEL.receive().await;
        //info!("received msg");
        let daqs = DAQ_CHANNEL.receive().await;
        
        let output = to_vec::<DAQSample, {64 as usize}>(&daqs);
        let op = output.unwrap();
        let restruct = from_bytes::<DAQSample>(op.deref());
        let re = restruct.unwrap();
        //println!("core 1 received: {:?}", re);
        println!("core 1 received from core 0: {:?}", re);

        //match CHANNEL.receive().await {
        //    LedState::On => led.set_high(),
        //    LedState::Off => led.set_low(),
        //}
    }
}