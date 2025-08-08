use defmt::*;
use embassy_rp::{
    gpio::{AnyPin, Input, Level, Output, Pull},
    pwm::{Config as PWMConfig, Pwm, SetDutyCycle},
    spi::{Async, Config as SPIConfig, Instance, Phase, Polarity, Spi},
};
use embassy_time::Timer;

use crate::data::DAQSample;

pub struct DAQFpga<'a, T>
where
    T: Instance,
{
    spi: Spi<'a, T, Async>,

    cs: Output<'a>,
    creset: Output<'a>,
    cdone: Input<'a>,

    mcu_clk: Pwm<'a>,
    pll_lock: Input<'a>,

    reset_pin: Output<'a>,
    interrupt: Input<'a>,

    read_buffer: [u8; 16],
}

pub fn daq_fpga_spi_config() -> SPIConfig {
    let mut config = SPIConfig::default();
    config.frequency = 5_000_000;
    config.polarity = Polarity::IdleHigh;
    config.phase = Phase::CaptureOnSecondTransition;
    config
}

pub fn daq_fpga_clock_config() -> PWMConfig {
    let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
    let divider = 1u8;
    let period = (clock_freq_hz / (10_000_000 * divider as u32)) as u16 - 1;

    let mut c = PWMConfig::default();
    c.top = period;
    c.divider = divider.into();

    c
}

impl<'a, T> DAQFpga<'a, T>
where
    T: Instance,
{
    pub fn new(
        spi_bus: Spi<'a, T, Async>,

        cs: AnyPin,
        creset: AnyPin,
        cdone: AnyPin,

        mcu_clk: Pwm<'a>,
        pll_lock: AnyPin,

        reset_pin: AnyPin,
        interrupt: AnyPin,
    ) -> Self {
        Self {
            spi: spi_bus,

            cs: Output::new(cs, Level::Low),
            creset: Output::new(creset, Level::Low),
            cdone: Input::new(cdone, Pull::None),

            mcu_clk: mcu_clk,
            pll_lock: Input::new(pll_lock, Pull::Down),

            reset_pin: Output::new(reset_pin, Level::High),
            interrupt: Input::new(interrupt, Pull::None),

            read_buffer: [0u8; 16],
        }
    }

    pub async fn configure(&mut self, bitstream: &'static [u8]) -> Result<(), ()> {
        if self.cdone.is_low() {
            info!("config proceed");
        } else {
            info!("config err, cdone already high");
            return Err(());
        }

        self.creset.set_high();
        Timer::after_micros(1200).await;

        self.cs.set_high();
        self.spi.write(&[0]).await.map_err(|_| ())?;
        info!("ok 8 dummy");
        self.cs.set_low();

        info!("bitstream size: {}", bitstream.len());
        self.spi.write(bitstream).await.map_err(|_| ())?;
        info!("ok bitstream");
        self.cs.set_high();

        if self.cdone.is_high() {
            info!("last 49(56) cycles");
            self.spi.write(&[0xFFu8; 7]).await.map_err(|_| ())?;
            info!("last config 56 bits ok");
            info!("confirm config done");
        } else {
            warn!("cdone never high");
            return Err(());
        }

        Ok(())
    }

    pub async fn setup_clocks(&mut self) -> Result<(), ()> {
        self.mcu_clk.set_duty_cycle_percent(50).map_err(|_| ())?;

        while self.pll_lock.is_low() {
            warn!("pll not locked");
            Timer::after_millis(500).await;
        }
        info!("pll locked");

        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), ()> {
        self.reset_pin.set_low();
        self.reset_pin.set_high();

        match self.interrupt.is_low() {
            true => {
                info!("reset success");
                Ok(())
            }
            false => {
                warn!("reset failed");
                Err(())
            }
        }
    }

    pub async fn await_sample(&mut self) {
        self.interrupt.wait_for_falling_edge().await;
    }

    pub fn read_sample(&mut self) -> Result<DAQSample, ()> {
        self.cs.set_low();
        self.spi
            .blocking_read(&mut self.read_buffer)
            .map_err(|_| ())?;
        self.cs.set_high();

        let (start_byte, end_byte) = (self.read_buffer[0], self.read_buffer[15]);
        if start_byte != 0x7E {
            warn!("start byte error");
            return Err(());
        }
        if end_byte != 0x7D {
            warn!("end byte error");
            return Err(());
        }

        let trigger_id_buf = &self.read_buffer[1..3];
        let trigger_clk_buf = &self.read_buffer[3..11];
        let trigger_data_buf = &self.read_buffer[11..15];

        let trigger_id = u16::from_be_bytes(trigger_id_buf.try_into().unwrap());
        let trigger_clk = u64::from_be_bytes(trigger_clk_buf.try_into().unwrap());
        let data_clk_buf = u32::from_be_bytes(trigger_data_buf.try_into().unwrap());
        let trigger_data = data_clk_buf & 0x00FF_FFFF;
        let veto_in = (data_clk_buf >> 31 & 1) != 0;
        let internal_trigger =(data_clk_buf >> 30 & 1) != 0;

        Ok(DAQSample {
            trigger_id: trigger_id,
            trigger_clk: trigger_clk,
            trigger_data: trigger_data,
            veto_in: veto_in,
            internal_trigger: internal_trigger
        })
    }
}
