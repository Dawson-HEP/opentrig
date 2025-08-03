//! This example shows powerful PIO module in the RP2040 chip.

#![no_std]
#![no_main]
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, Peripheral};
use embassy_rp::clocks::clk_sys_freq;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{Config, Direction, InterruptHandler, Pio, ShiftConfig, ShiftDirection};
use embassy_time::Timer;
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use {defmt_rtt as _, panic_probe as _};

use pio::{Assembler, MovSource, OutDestination, SetDestination, SideSet, RP2040_MAX_PROGRAM_SIZE};

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

    let mut a = Assembler::<{ RP2040_MAX_PROGRAM_SIZE }>::new_with_side_set(SideSet::new(false, 2, false));
    // let mut a = Assembler::<{ RP2040_MAX_PROGRAM_SIZE }>::new();
    let mut loop_label = a.label();
    let mut bitloop = a.label();

    a.bind(&mut loop_label);
    a.pull_with_side_set(false, false, 0b11);
    a.nop_with_side_set(0b11);
    a.nop_with_side_set(0b10);
    a.set_with_side_set(SetDestination::X, 31, 0b10);

    a.bind(&mut bitloop);
    a.out_with_side_set(OutDestination::PINS, 1, 0b01);
    a.nop_with_side_set(0b01);
    a.nop_with_side_set(0b00);
    a.jmp_with_side_set(pio::JmpCondition::XDecNonZero, &mut bitloop, 0b00);

    let pio_prg = a.assemble_program();

    let mut cfg = Config::default();
    let (clk, trig, trig_id) = (
        common.make_pio_pin(p.PIN_0),
        common.make_pio_pin(p.PIN_1),
        common.make_pio_pin(p.PIN_2),
    );

    cfg.use_program(&common.load_program(&pio_prg), &[&clk, &trig]);
    cfg.clock_divider = (U56F8!(125_000_000) / U56F8!(1_000_000)).to_fixed();
    cfg.shift_in = ShiftConfig {
        auto_fill: false,
        threshold: 32,
        direction: ShiftDirection::Left,
    };
    cfg.shift_out = ShiftConfig {
        auto_fill: false,
        threshold: 32,
        direction: ShiftDirection::Left,
    };

    cfg.set_out_pins(&[&trig_id]);

    sm.set_pin_dirs(Direction::Out, &[&clk, &trig, &trig_id]);
    sm.set_config(&cfg);
    sm.set_enable(true);

    let mut dma_out_ref = p.DMA_CH0.into_ref();
    // let mut dma_in_ref = p.DMA_CH1;

    let test = [290138920u32; 1];

    loop {
        let (rx, tx) = sm.rx_tx();
        tx.dma_push(dma_out_ref.reborrow(), &test, false).await;
        info!("pushed to dma");

        Timer::after_millis(200).await;
    }
}
