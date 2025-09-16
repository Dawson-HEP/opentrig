//! This module is meant to manage 6 MCP4728 DACs that share a common I2C line,
//! using the mcp4728 crate.
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

/// Error returned by DacManager.
#[derive(Debug)]
pub enum DacError {
    InvalidDacId(usize),
    InputVoltageOutOfBounds(u16),
    McpError(mcp4728::Error<I2cDeviceError<embassy_rp::i2c::Error>>),
}

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

/// Manager for 6 MCP4728 DACs.
pub struct DacManager<'a> {
    pub dacs: [MCP4728Async<I2cDevice<'a, NoopRawMutex, I2c<'static, I2C0, i2c::Async>>>; 6],
    pub ldacs: [Output<'a>; 6],
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
        for i in 0..6 {
            self.dacs[i]
                .write_voltage_reference_mode(vref, vref, vref, vref)
                .await
                .map_err(|e| DacError::McpError(e))
                .unwrap();
            self.dacs[i]
                .write_gain_mode(gain, gain, gain, gain)
                .await
                .map_err(|e| DacError::McpError(e))
                .unwrap();
            self.dacs[i]
                .write_power_down_mode(power, power, power, power)
                .await
                .map_err(|e| DacError::McpError(e))
                .unwrap();
        }

        info!("All MCPs set to default modes.");

        Ok(())
    }

    /// Get the DAC corresponding to a specific ID.
    pub async fn get_dac(
        &mut self,
        dac_id: usize,
    ) -> Result<
        &mut MCP4728Async<I2cDevice<'a, NoopRawMutex, I2c<'static, I2C0, i2c::Async>>>,
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
        // Read the registers of the correct DAC.
        let dac = self.get_dac(dac_id).await.unwrap();
        let registers: Registers = dac.read().await.map_err(|e| DacError::McpError(e)).unwrap();

        // Return the specified channel.
        Ok(match channel {
            mcp4728::Channel::A => registers.channel_a_input.channel_state,
            mcp4728::Channel::B => registers.channel_b_input.channel_state,
            mcp4728::Channel::C => registers.channel_c_input.channel_state,
            mcp4728::Channel::D => registers.channel_d_input.channel_state,
        })
    }

    /// Change the voltage on a single channel to the specified value in mv.
    pub async fn set_voltage(
        &mut self,
        dac_id: usize,
        channel: mcp4728::Channel,
        voltage: u16,
    ) -> Result<(), DacError> {
        let channel_state = self.read_channel(dac_id, channel).await.unwrap();
        let gain_mode = channel_state.gain_mode;
        let input = match gain_mode {
            GainMode::TimesOne => voltage * 2,
            GainMode::TimesTwo => voltage,
        };

        if input > 4096 {
            return Err(DacError::InputVoltageOutOfBounds(input));
        }

        // Create the new data.
        let mut channel_state = self.read_channel(dac_id, channel).await.unwrap().clone();
        channel_state.value = input;

        // Get referenced DAC.
        let dac = self.get_dac(dac_id).await.unwrap();

        // Write the data.
        dac.single_write(channel, mcp4728::OutputEnableMode::Update, &channel_state)
            .await
            .map_err(|e| DacError::McpError(e))
            .unwrap();

        info!("Voltage changed successfully.");
        Ok(())
    }

    /// Change the voltage reference mode on a single channel.
    pub async fn set_vref_mode(
        &mut self,
        dac_id: usize,
        channel: mcp4728::Channel,
        mode: mcp4728::VoltageReferenceMode,
    ) -> Result<(), DacError> {
        // Create the new data.
        let mut channel_state = self.read_channel(dac_id, channel).await.unwrap().clone();
        channel_state.voltage_reference_mode = mode;

        // Get referenced DAC.
        let dac = self.get_dac(dac_id).await.unwrap();

        // Write the data.
        dac.single_write(channel, mcp4728::OutputEnableMode::Update, &channel_state)
            .await
            .map_err(|e| DacError::McpError(e))
            .unwrap();

        info!("Vref mode changed successfully.");
        Ok(())
    }

    /// Change the gain mode on a single channel.
    pub async fn set_gain_mode(
        &mut self,
        dac_id: usize,
        channel: mcp4728::Channel,
        mode: mcp4728::GainMode,
    ) -> Result<(), DacError> {
        // Create the new data.
        let mut channel_state = self.read_channel(dac_id, channel).await.unwrap().clone();
        channel_state.gain_mode = mode;

        // Get referenced DAC.
        let dac = self.get_dac(dac_id).await.unwrap();

        // Write the data.
        dac.single_write(channel, mcp4728::OutputEnableMode::Update, &channel_state)
            .await
            .map_err(|e| DacError::McpError(e))
            .unwrap();

        info!("Gain mode changed successfully.");
        Ok(())
    }

    /// Change the gain mode on a single channel.
    pub async fn set_power_down_mode(
        &mut self,
        dac_id: usize,
        channel: mcp4728::Channel,
        mode: mcp4728::PowerDownMode,
    ) -> Result<(), DacError> {
        // Create the new data.
        let mut channel_state = self.read_channel(dac_id, channel).await.unwrap().clone();
        channel_state.power_down_mode = mode;

        // Get referenced DAC.
        let dac = self.get_dac(dac_id).await.unwrap();

        // Write the data.
        dac.single_write(channel, mcp4728::OutputEnableMode::Update, &channel_state)
            .await
            .map_err(|e| DacError::McpError(e))
            .unwrap();

        info!("Power down mode changed successfully.");
        Ok(())
    }

    /// Change the voltage on all 24 channels to the specified value in mv.
    pub async fn set_all_voltages(&mut self, voltages: [u16; 24]) -> Result<(), DacError> {
        // Loop through each DAC.
        for i in 0..6 {
            let j = i * 4;

            // Read the gain mode of each channel.
            let registers = self.dacs[i]
                .read()
                .await
                .map_err(|e| DacError::McpError(e))
                .unwrap();
            let gains = [
                registers.channel_a_input.channel_state.gain_mode,
                registers.channel_b_input.channel_state.gain_mode,
                registers.channel_c_input.channel_state.gain_mode,
                registers.channel_d_input.channel_state.gain_mode,
            ];

            // Convert the the user input in mv, to the DAC register input.
            let mut inputs: [u16; 4] = [0; 4];
            for k in 0..4 {
                inputs[k] = if gains[k] == GainMode::TimesOne {
                    // Gain 1.
                    voltages[j + k] * 2
                } else {
                    // Gain 2.
                    voltages[j + k]
                };
                if inputs[k] > 4096 {
                    return Err(DacError::InputVoltageOutOfBounds(inputs[k]));
                };
            }

            // Write values to the DAC.
            self.dacs[i]
                .fast_write(inputs[0], inputs[1], inputs[2], inputs[3])
                .await
                .map_err(|e| DacError::McpError(e))
                .unwrap();
        }

        info!("All voltages changed successfully.");
        Ok(())
    }

    /// Change the voltage reference mode on all 24 channels.
    pub async fn set_all_vref_modes(
        &mut self,
        modes: [mcp4728::VoltageReferenceMode; 24],
    ) -> Result<(), DacError> {
        for i in 0..6 {
            let j = i * 4;

            self.dacs[i]
                .write_voltage_reference_mode(modes[j], modes[j + 1], modes[j + 2], modes[j + 3])
                .await
                .map_err(|e| DacError::McpError(e))
                .unwrap();
        }

        info!("All vref modes changed successfully.");
        Ok(())
    }

    /// Change the gain mode on all 24 channels.
    pub async fn set_all_gain_modes(
        &mut self,
        modes: [mcp4728::GainMode; 24],
    ) -> Result<(), DacError> {
        for i in 0..6 {
            let j = i * 4;

            self.dacs[i]
                .write_gain_mode(modes[j], modes[j + 1], modes[j + 2], modes[j + 3])
                .await
                .map_err(|e| DacError::McpError(e))
                .unwrap();
        }

        info!("All gain modes changed successfully.");
        Ok(())
    }

    /// Change the power down mode on all 24 channels.
    pub async fn set_all_power_down_modes(
        &mut self,
        modes: [mcp4728::PowerDownMode; 24],
    ) -> Result<(), DacError> {
        for i in 0..6 {
            let j = i * 4;

            self.dacs[i]
                .write_power_down_mode(modes[j], modes[j + 1], modes[j + 2], modes[j + 3])
                .await
                .map_err(|e| DacError::McpError(e))
                .unwrap();
        }

        info!("All power down modes changed successfully.");
        Ok(())
    }
}
