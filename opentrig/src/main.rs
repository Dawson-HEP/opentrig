//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

use crate::fpga::{DAQFpga, daq_fpga_clock_config, daq_fpga_spi_config};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{gpio::Pin, pwm::Pwm, spi::Spi};

use {defmt_rtt as _, panic_probe as _};

use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_usb::driver::{Endpoint, EndpointIn, EndpointOut};

use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_usb::msos::{self, windows_version};
use embassy_usb::{Builder, Config};
use {defmt_rtt as _, panic_probe as _};

// This is a randomly generated GUID to allow clients on Windows to find our device
const DEVICE_INTERFACE_GUIDS: &[&str] = &["{AFB9A6FB-30BA-44BC-9232-806CFC875321}"];

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

mod data;
mod fpga;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Create the driver, from the HAL.
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
    let mut read_ep = alt.endpoint_bulk_out(64);
    let mut write_ep = alt.endpoint_bulk_in(64);
    drop(function);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    let (rx, tx, clk) = (p.PIN_20, p.PIN_19, p.PIN_18);
    let spi_config = daq_fpga_spi_config();
    let spi = Spi::new(p.SPI0, clk, tx, rx, p.DMA_CH0, p.DMA_CH1, spi_config);

    let pwm_config = daq_fpga_clock_config();
    let fpga_mcu_clk = Pwm::new_output_b(p.PWM_SLICE5, p.PIN_27, pwm_config);

    let mut daq = DAQFpga::new(
        spi,
        p.PIN_17.degrade(),
        p.PIN_13.degrade(),
        p.PIN_14.degrade(),
        fpga_mcu_clk,
        p.PIN_26.degrade(),
        p.PIN_15.degrade(),
        p.PIN_16.degrade(),
    );

    daq.configure(include_bytes!("fpga/main.bin"))
        .await
        .unwrap();
    daq.setup_clocks().await.unwrap();

    daq.reset().unwrap();

    let main_loop = async {
        loop {
            read_ep.wait_enabled().await;
            info!("usb connected");
            loop {
                daq.await_sample().await;

                if let Ok(sample) = daq.read_sample() {
                    // let d = sample.trigger_data;

                    // info!(
                    //     "trigger_id {}, trigger_clk {}, trigger_data [0b{:08b} 0b{:08b} 0b{:08b} 0b{:08b}], veto_in {}, internal_trigger {}",
                    //     sample.trigger_id,
                    //     sample.trigger_clk,
                    //     (d >> 24) & 0xFF,
                    //     (d >> 16) & 0xFF,
                    //     (d >> 8) & 0xFF,
                    //     d & 0xFF,
                    //     sample.veto_in,
                    //     sample.internal_trigger,
                    // );

                    write_ep.write(daq.last_sample_bytes()).await.ok();
                }
            }
        }
    };

    join(usb_fut,main_loop).await;
}
