//! This example shows powerful PIO module in the RP2040 chip.

#![no_std]
#![no_main]
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, Peripheral};
use embassy_rp::clocks::clk_sys_freq;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{Config, Direction, FifoJoin, InterruptHandler, Pio, ShiftConfig, ShiftDirection};
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

    let mut a = Assembler::<{ RP2040_MAX_PROGRAM_SIZE }>::new_with_side_set(SideSet::new(false, 1, false));
    // let mut a = Assembler::<{ RP2040_MAX_PROGRAM_SIZE }>::new();
    let mut loop_label = a.label();
    let mut bitloop = a.label();

    a.bind(&mut loop_label);
    a.pull_with_side_set(false, false, 1);
    a.nop_with_side_set(1);
    a.nop_with_side_set(0);
    a.set_with_side_set(SetDestination::X, 31, 0);

    a.bind(&mut bitloop);
    a.out_with_side_set(OutDestination::PINS, 2, 1);
    a.nop_with_side_set(1);
    a.nop_with_side_set(0);
    a.jmp_with_side_set(pio::JmpCondition::XDecNonZero, &mut bitloop, 0);

    let pio_prg = a.assemble_program();

    let mut cfg = Config::default();
    let (clk, trig, trig_id) = (
        common.make_pio_pin(p.PIN_0),
        common.make_pio_pin(p.PIN_1),
        common.make_pio_pin(p.PIN_2),
    );

    cfg.use_program(&common.load_program(&pio_prg), &[&clk]);
    cfg.clock_divider = (U56F8!(125_000_000) / U56F8!(1_000_000)).to_fixed();
    cfg.shift_out = ShiftConfig {
        auto_fill: false,
        threshold: 32,
        direction: ShiftDirection::Left,
    };
    cfg.fifo_join = FifoJoin::TxOnly;

    cfg.set_set_pins(&[&trig, &trig_id]);
    cfg.set_out_pins(&[&trig, &trig_id]);

    sm.set_pin_dirs(Direction::Out, &[&clk, &trig, &trig_id]);
    sm.set_config(&cfg);
    sm.set_enable(true);

    let mut dma_out_ref = p.DMA_CH0.into_ref();
    // let mut dma_in_ref = p.DMA_CH1;

    let trigger_id = 0xA111u16;

    let encoded = encode(trigger_id);
    let mut test = [0u32; 100];
    test[0] = encoded;

    loop {
        let (rx, tx) = sm.rx_tx();
        tx.dma_push(dma_out_ref.reborrow(), &test, false).await;
    }
}

fn encode(x: u16) -> u32 {
    let mut result = 0;
    for i in 0..16 {
        let bit = (x >> i) & 1;
        result |= (bit as u32) << (i * 2);
    }
    result >> 1 | 1u32 << 30
}
