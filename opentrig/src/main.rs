//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{self, Input, Pull},
    spi::{Config, Phase, Polarity, Spi},
};
use embassy_time::Timer;
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut creset = Output::new(p.PIN_13, Level::High);

    creset.set_slew_rate(gpio::SlewRate::Fast);
    creset.set_drive_strength(gpio::Drive::_12mA);
    let mut cs = Output::new(p.PIN_17, Level::High);
    let cdone = Input::new(p.PIN_14, Pull::Up);
    let (rx, tx, clk) = (p.PIN_20, p.PIN_19, p.PIN_18);

    let mut config = Config::default();
    config.frequency = 10_000_000;
    config.polarity = Polarity::IdleHigh;
    config.phase = Phase::CaptureOnSecondTransition;

    let mut fpga_spi = Spi::new(p.SPI0, clk, tx, rx, p.DMA_CH0, p.DMA_CH1, config);

    match cdone.is_low() {
        true => info!("config proceed"),
        false => info!("config err, cdone"),
    }

    cs.set_low();
    creset.set_low();
    Timer::after_micros(100).await;
    creset.set_high();
    Timer::after_micros(1200).await;

    cs.set_high();
    match fpga_spi.blocking_write(&[0]) {
        Err(_) => info!("err"),
        Ok(()) => info!("ok 8 dummy"),
    }
    cs.set_low();

    let bitstream = include_bytes!("fpga/main.bin");
    match fpga_spi.write(bitstream).await {
        Err(_) => info!("err"),
        Ok(()) => info!("ok bitstream"),
    }

    cs.set_high();

    let mut cdone_await_clk_count = 0;
    while cdone.is_low() && cdone_await_clk_count <= 100 {
        match fpga_spi.blocking_write(&[0]) {
            Err(_) => info!("err"),
            Ok(()) => info!("write clk ok"),
        }
        cdone_await_clk_count += 8;
    }
    if cdone_await_clk_count > 100 {
        warn!("config clk count err, cdone not high");
    }

    if cdone.is_low() {
        match fpga_spi.blocking_write(&[0xFFu8; 7]) {
            Err(_) => info!("last config 56 bits err"),
            Ok(()) => info!("last config 56 bits ok"),
        }
    }

    if cdone.is_high() {
        info!("confirm config done");
    }

    loop {}
}
