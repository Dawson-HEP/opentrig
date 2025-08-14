//! This module is meant to manage 6 MCP4728 DACs that share a common I2C line.
//!
//! Some functionalities are left to be handled by the mcp4728 crate itself:
//! Functions like MCP4728Async.fast_write() are not cloned in this module, users
//! should access the MCP4728Async instances whithin the DacManager.dacs field.

use defmt::*;
use embassy_embedded_hal::shared_bus::{I2cDeviceError, asynch::i2c::I2cDevice};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::Output;
use embassy_rp::i2c::{self, I2c, InterruptHandler};
use embassy_rp::peripherals::I2C0;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use mcp4728::{GainMode, MCP4728Async, PowerDownMode, Registers};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<I2C0>;
});

/// Returns a static initialized I2c bus wrapped in a Mutex.
fn init_i2c(
    i2c_peri: I2C0,
    scl: impl i2c::SclPin<I2C0>,
    sda: impl i2c::SdaPin<I2C0>,
) -> &'static Mutex<NoopRawMutex, I2c<'static, I2C0, i2c::Async>> {
    // Initialize bus.
    let i2c = I2c::new_async(i2c_peri, scl, sda, Irqs, i2c::Config::default());

    // Wrap bus.
    static I2C_BUS: StaticCell<Mutex<NoopRawMutex, I2c<'static, I2C0, i2c::Async>>> =
        StaticCell::new();
    I2C_BUS.init(Mutex::new(i2c))
}

/// Error returned by DacManager.
#[derive(Debug)]
pub enum DacError {
    InvalidDacId(usize),
    McpError(mcp4728::Error<I2cDeviceError<embassy_rp::i2c::Error>>),
}

/// Manager for 6 MCP4728 DACs.
pub struct DacManager<'a> {
    dacs: [MCP4728Async<I2cDevice<'a, NoopRawMutex, I2c<'static, I2C0, i2c::Async>>>; 6],
    ldacs: [Output<'a>; 6],
}

impl<'a> DacManager<'a> {
    /// Create a new DacManager instance.
    pub fn new(
        i2c_peri: I2C0,
        scl: impl i2c::SclPin<I2C0>,
        sda: impl i2c::SdaPin<I2C0>,
        ldacs: [Output<'a>; 6],
    ) -> Self {
        let i2c_bus = init_i2c(i2c_peri, scl, sda);

        Self {
            dacs: [
                MCP4728Async::new(I2cDevice::new(i2c_bus), 0x60),
                MCP4728Async::new(I2cDevice::new(i2c_bus), 0x61),
                MCP4728Async::new(I2cDevice::new(i2c_bus), 0x62),
                MCP4728Async::new(I2cDevice::new(i2c_bus), 0x63),
                MCP4728Async::new(I2cDevice::new(i2c_bus), 0x64),
                MCP4728Async::new(I2cDevice::new(i2c_bus), 0x65),
            ],
            ldacs: ldacs,
        }
    }

    /// Set all DACs to their default vref, gain and powerdown modes.
    pub async fn init(&mut self) -> Result<(), DacError> {
        // Get default values.
        let vref = mcp4728::VoltageReferenceMode::Internal;
        let gain = GainMode::TimesOne;
        let power = PowerDownMode::Normal;

        // Loop through DACs to write defaults.
        for dac in self.dacs.iter_mut() {
            dac.write_voltage_reference_mode(vref, vref, vref, vref)
                .await
                .map_err(|e| DacError::McpError(e))
                .unwrap();
            dac.write_gain_mode(gain, gain, gain, gain)
                .await
                .map_err(|e| DacError::McpError(e))
                .unwrap();
            dac.write_power_down_mode(power, power, power, power)
                .await
                .map_err(|e| DacError::McpError(e))
                .unwrap();
        }

        Ok(())
    }

    /// Change the voltage on a single channel.
    pub async fn set_voltage(
        &mut self,
        dac_id: usize,
        channel: mcp4728::Channel,
        voltage: u16,
    ) -> Result<(), DacError> {
        // Handle ValueOutOfBounds error.
        if dac_id > 5 {
            return Err(DacError::InvalidDacId(dac_id));
        }

        let dac: &mut MCP4728Async<I2cDevice<'a, NoopRawMutex, I2c<'static, I2C0, i2c::Async>>> =
            &mut self.dacs[dac_id];

        let registers: Registers = dac.read().await.map_err(|e| DacError::McpError(e)).unwrap();

        // Read the specified channel.
        let mut channel_state = match channel {
            mcp4728::Channel::A => registers.channel_a_input.channel_state,
            mcp4728::Channel::B => registers.channel_b_input.channel_state,
            mcp4728::Channel::C => registers.channel_c_input.channel_state,
            mcp4728::Channel::D => registers.channel_d_input.channel_state,
        };

        channel_state.value = voltage;
        dac.single_write(channel, mcp4728::OutputEnableMode::Update, &channel_state);

        Ok(())
    }

    /// Change the voltage on all 24 channels.
    pub async fn set_all_voltages(&mut self, voltages: [u16; 24]) -> Result<(), DacError> {
        for (i, dac) in self.dacs.iter_mut().enumerate() {
            let j = i * 4;

            dac.fast_write(
                voltages[j],
                voltages[j + 1],
                voltages[j + 2],
                voltages[j + 3],
            )
            .await
            .map_err(|e| DacError::McpError(e))
            .unwrap();
        }

        Ok(())
    }
}
