//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.
#![allow(warnings)] 
#![no_std]
#![no_main]

use crate::{dac::DacError, fpga::{daq_fpga_clock_config, daq_fpga_spi_config, DAQFpga}};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{gpio::Pin, pwm::Pwm, spi::Spi};

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