//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

use crate::dac::DacManager;
use crate::fpga::{DAQFpga, daq_fpga_clock_config, daq_fpga_spi_config};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output, Pin};
use embassy_rp::{pwm::Pwm, spi::Spi};

use {defmt_rtt as _, panic_probe as _};

mod dac;
mod data;
mod fpga;

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

    let mut dac_manager = DacManager::new(p.I2C0, p.PIN_1, p.PIN_0, ldacs);
    dac_manager.init().await.unwrap();
    dac_manager.set_all_voltages([1200; 24]).await.unwrap();

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

    loop {
        daq.await_sample().await;
        if let Ok(sample) = daq.read_sample() {
            info!(
                "trigger_id {}, trigger_clk {}, trigger_data {}, veto_in {}, internal_trigger {}",
                sample.trigger_id,
                sample.trigger_clk,
                sample.trigger_data,
                sample.veto_in,
                sample.internal_trigger,
            );
        }
    }
}
