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
use embassy_rp::peripherals::I2C1;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use mcp4728::{GainMode, MCP4728Async, PowerDownMode, Registers};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

/// Error returned by DacManager.
#[derive(Debug)]
pub enum DacError {
    InvalidDacId(usize),
    McpError(mcp4728::Error<I2cDeviceError<embassy_rp::i2c::Error>>),
}

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});

/// Returns a static initialized I2c bus wrapped in a Mutex.
fn init_i2c(
    i2c_peri: I2C1,
    scl: impl i2c::SclPin<I2C1>,
    sda: impl i2c::SdaPin<I2C1>,
) -> &'static Mutex<NoopRawMutex, I2c<'static, I2C1, i2c::Async>> {
    // Initialize bus.
    let i2c = I2c::new_async(i2c_peri, scl, sda, Irqs, i2c::Config::default());

    // Wrap bus.
    static I2C_BUS: StaticCell<Mutex<NoopRawMutex, I2c<'static, I2C1, i2c::Async>>> =
        StaticCell::new();
    I2C_BUS.init(Mutex::new(i2c))
}

/// Manager for 6 MCP4728 DACs.
pub struct DacManager<'a> {
    dacs: [MCP4728Async<I2cDevice<'a, NoopRawMutex, I2c<'static, I2C1, i2c::Async>>>; 6],
    ldacs: [Output<'a>; 6],
}

impl<'a> DacManager<'a> {
    /// Create a new DacManager instance.
    pub fn new(
        i2c_peri: I2C1,
        scl: impl i2c::SclPin<I2C1>,
        sda: impl i2c::SdaPin<I2C1>,
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

    /// Get the DAC corresponding to a specific ID.
    pub async fn get_dac(
        &mut self,
        dac_id: usize,
    ) -> Result<
        &mut MCP4728Async<I2cDevice<'a, NoopRawMutex, I2c<'static, I2C1, i2c::Async>>>,
        DacError,
    > {
        // Handle ValueOutOfBounds error.
        if dac_id > 5 {
            return Err(DacError::InvalidDacId(dac_id));
        }
        Ok(&mut self.dacs[dac_id])
    }

    /// Read the data from the specified MCP4728 on the specified channel.
    pub async fn read_channel(
        &mut self,
        dac_id: usize,
        channel: mcp4728::Channel,
    ) -> Result<mcp4728::ChannelState, DacError> {
        let dac = self.get_dac(dac_id).await.unwrap();
        let registers: Registers = dac.read().await.map_err(|e| DacError::McpError(e)).unwrap();

        // Read and return the specified channel.
        Ok(match channel {
            mcp4728::Channel::A => registers.channel_a_input.channel_state,
            mcp4728::Channel::B => registers.channel_b_input.channel_state,
            mcp4728::Channel::C => registers.channel_c_input.channel_state,
            mcp4728::Channel::D => registers.channel_d_input.channel_state,
        })
    }

    /// Change the voltage on a single channel.
    pub async fn set_voltage(
        &mut self,
        dac_id: usize,
        channel: mcp4728::Channel,
        voltage: u16,
    ) -> Result<(), DacError> {
        // Get new data.
        let mut channel_state = self.read_channel(dac_id, channel).await.unwrap().clone();
        channel_state.value = voltage;

        // Get referenced DAC.
        let dac = self.get_dac(dac_id).await.unwrap();

        // Write the data.
        dac.single_write(channel, mcp4728::OutputEnableMode::Update, &channel_state);

        Ok(())
    }

    /// Change the voltage reference mode on a single channel.
    pub async fn set_vref_mode(
        &mut self,
        dac_id: usize,
        channel: mcp4728::Channel,
        mode: mcp4728::VoltageReferenceMode,
    ) -> Result<(), DacError> {
        // Get new data.
        let mut channel_state = self.read_channel(dac_id, channel).await.unwrap().clone();
        channel_state.voltage_reference_mode = mode;

        // Get referenced DAC.
        let dac = self.get_dac(dac_id).await.unwrap();

        // Write the data.
        dac.single_write(channel, mcp4728::OutputEnableMode::Update, &channel_state);

        Ok(())
    }

    /// Change the gain mode on a single channel.
    pub async fn set_gain_mode(
        &mut self,
        dac_id: usize,
        channel: mcp4728::Channel,
        mode: mcp4728::GainMode,
    ) -> Result<(), DacError> {
        // Get new data.
        let mut channel_state = self.read_channel(dac_id, channel).await.unwrap().clone();
        channel_state.gain_mode = mode;

        // Get referenced DAC.
        let dac = self.get_dac(dac_id).await.unwrap();

        // Write the data.
        dac.single_write(channel, mcp4728::OutputEnableMode::Update, &channel_state);

        Ok(())
    }

    /// Change the gain mode on a single channel.
    pub async fn set_power_down_mode(
        &mut self,
        dac_id: usize,
        channel: mcp4728::Channel,
        mode: mcp4728::PowerDownMode,
    ) -> Result<(), DacError> {
        // Get new data.
        let mut channel_state = self.read_channel(dac_id, channel).await.unwrap().clone();
        channel_state.power_down_mode = mode;

        // Get referenced DAC.
        let dac = self.get_dac(dac_id).await.unwrap();

        // Write the data.
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

    /// Change the voltage on all 24 channels.
    pub async fn set_all_vref_modes(
        &mut self,
        modes: [mcp4728::VoltageReferenceMode; 24],
    ) -> Result<(), DacError> {
        for (i, dac) in self.dacs.iter_mut().enumerate() {
            let j = i * 4;

            dac.write_voltage_reference_mode(modes[j], modes[j + 1], modes[j + 2], modes[j + 3])
                .await
                .map_err(|e| DacError::McpError(e))
                .unwrap();
        }

        Ok(())
    }

    /// Change the voltage on all 24 channels.
    pub async fn set_all_gain_modes(
        &mut self,
        modes: [mcp4728::GainMode; 24],
    ) -> Result<(), DacError> {
        for (i, dac) in self.dacs.iter_mut().enumerate() {
            let j = i * 4;

            dac.write_gain_mode(modes[j], modes[j + 1], modes[j + 2], modes[j + 3])
                .await
                .map_err(|e| DacError::McpError(e))
                .unwrap();
        }

        Ok(())
    }

    /// Change the voltage on all 24 channels.
    pub async fn set_all_power_down_modes(
        &mut self,
        modes: [mcp4728::PowerDownMode; 24],
    ) -> Result<(), DacError> {
        for (i, dac) in self.dacs.iter_mut().enumerate() {
            let j = i * 4;

            dac.write_power_down_mode(modes[j], modes[j + 1], modes[j + 2], modes[j + 3])
                .await
                .map_err(|e| DacError::McpError(e))
                .unwrap();
        }

        Ok(())
    }
}
