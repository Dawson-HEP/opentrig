//! This example shows powerful PIO module in the RP2040 chip.

#![no_std]
#![no_main]
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::clocks::clk_sys_freq;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{
    Config, Direction, FifoJoin, InterruptHandler, Pio, PioPin, ShiftConfig, ShiftDirection, StatusSource,
};
use embassy_rp::{Peripheral, bind_interrupts};
use embassy_time::{Duration, Ticker, Timer};
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use {defmt_rtt as _, panic_probe as _};

use heapless::Vec;

use pio::{Assembler, MovSource, OutDestination, RP2040_MAX_PROGRAM_SIZE, SetDestination, SideSet};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let pio = p.PIO0;
    let Pio {
        mut common,
        sm0: mut sm,
        ..
    } = Pio::new(pio, Irqs);

    let mut a =
        Assembler::<{ RP2040_MAX_PROGRAM_SIZE }>::new_with_side_set(SideSet::new(false, 1, false));
    let mut loop_label = a.label();
    let mut bitloop_label = a.label();

    a.bind(&mut loop_label);
    a.mov_with_side_set(pio::MovDestination::X, pio::MovOperation::Invert, MovSource::STATUS, 1);
    a.jmp_with_side_set(pio::JmpCondition::XIsZero, &mut loop_label, 0);

    a.bind(&mut bitloop_label);
    a.out_with_side_set(OutDestination::PINS, 32, 1);
    a.jmp_with_side_set(pio::JmpCondition::OutputShiftRegisterNotEmpty, &mut bitloop_label, 0);

    let pio_prg = a.assemble_program();

    let mut cfg = Config::default();

    let pin0 = common.make_pio_pin(p.PIN_0);
    let pin1 = common.make_pio_pin(p.PIN_1);
    let pin2 = common.make_pio_pin(p.PIN_2);
    let pin3 = common.make_pio_pin(p.PIN_3);
    let pin4 = common.make_pio_pin(p.PIN_4);
    let pin5 = common.make_pio_pin(p.PIN_5);
    let pin6 = common.make_pio_pin(p.PIN_6);
    let pin7 = common.make_pio_pin(p.PIN_7);

    let pin8 = common.make_pio_pin(p.PIN_8);
    let pin9 = common.make_pio_pin(p.PIN_9);
    let pin10 = common.make_pio_pin(p.PIN_10);
    let pin11 = common.make_pio_pin(p.PIN_11);
    let pin12 = common.make_pio_pin(p.PIN_12);
    let pin13 = common.make_pio_pin(p.PIN_13);
    let pin14 = common.make_pio_pin(p.PIN_14);
    let pin15 = common.make_pio_pin(p.PIN_15);

    let pin16 = common.make_pio_pin(p.PIN_16);
    let pin17 = common.make_pio_pin(p.PIN_17);
    let pin18 = common.make_pio_pin(p.PIN_18);
    let pin19 = common.make_pio_pin(p.PIN_19);
    let pin20 = common.make_pio_pin(p.PIN_20);
    let pin21 = common.make_pio_pin(p.PIN_21);
    let pin22 = common.make_pio_pin(p.PIN_22);
    
    let pin23 = common.make_pio_pin(p.PIN_23); // low
    let pin24 = common.make_pio_pin(p.PIN_24); // low

    let pin25 = common.make_pio_pin(p.PIN_25);

    let pin26 = common.make_pio_pin(p.PIN_26); // trigger_id
    let pin27 = common.make_pio_pin(p.PIN_27); // trigger
    let clk = common.make_pio_pin(p.PIN_28);   // clk

    cfg.use_program(&common.load_program(&pio_prg), &[&clk]);
    cfg.clock_divider = (U56F8!(125_000_000) / U56F8!(1_000_000)).to_fixed();
    cfg.shift_out = ShiftConfig {
        auto_fill: true,
        threshold: 32,
        direction: ShiftDirection::Right,
    };
    cfg.fifo_join = FifoJoin::TxOnly;

    cfg.status_n = 1;
    cfg.status_sel = StatusSource::TxFifoLevel;

    cfg.set_out_pins(&[
        &pin0, &pin1, &pin2, &pin3, &pin4, &pin5, &pin6, &pin7, &pin8, &pin9, &pin10, &pin11,
        &pin12, &pin13, &pin14, &pin15, &pin16, &pin17, &pin18, &pin19, &pin20, &pin21, &pin22,
        &pin23, &pin24, &pin25, &pin26, &pin27,
    ]);
    sm.set_pin_dirs(
        Direction::Out,
        &[
            &pin0, &pin1, &pin2, &pin3, &pin4, &pin5, &pin6, &pin7, &pin8, &pin9, &pin10, &pin11,
            &pin12, &pin13, &pin14, &pin15, &pin16, &pin17, &pin18, &pin19, &pin20, &pin21, &pin22,
            &pin23, &pin24, &pin25, &pin26, &pin27, &clk,
        ],
    );

    sm.set_config(&cfg);
    sm.set_enable(true);

    let mut dma_out_ref = p.DMA_CH0.into_ref();
    let mut trigger_id_buffer = [0u32; 18];
    let tx = sm.tx();

    // let mut ticker = Ticker::every(Duration::from_hz(10_000));
    let mut ticker = Ticker::every(Duration::from_hz(50));
    let mut trig_id = 0u16;
    loop {
        encode_trigger_id(&mut trigger_id_buffer, trig_id);

        tx.dma_push(dma_out_ref.reborrow(), &trigger_id_buffer, false).await;

        trig_id = (trig_id + 1) % u16::MAX;
        ticker.next().await;
    }
}


fn encode_trigger_id(buffer: &mut [u32], id: u16) {
    buffer[0] = 1 << 27;                        // GPIO27 -> Trigger
    for i in 0..16 {
        let j = 15 - i;                  // Encode MSB-first
        let bit = (id >> j) as u32 & 1;
        buffer[i + 1] = bit << 26;              // GPIO26 -> Trigger ID
    }
}
