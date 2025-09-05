use crate::dac;
use crate::dac::DacManager;




async fn two_u8_into_u16(u8_higher_order:u8, u8_lower_order:u8) -> u16 {
    ((u8_higher_order as u16) << 8) | u8_lower_order as u16
}

pub async fn call_set_power_down_mode(inputs:&[u8], mut dac_manager:DacManager<'static>) {
    let input_dac_id  = inputs[1]; // usize
    let input_channel = inputs[2]; // mcp4728::Channel
    let input_power_down_mode = inputs[3]; // mcp4728::PowerDownMode

    let dac_id = match_dac_id(input_dac_id).await;
    let channel = match_channel(input_channel).await;
    let mode = match_power_down_mode(input_power_down_mode).await;

    if dac_id.is_some() && channel.is_some() && mode.is_some() {
        dac_manager.set_power_down_mode(dac_id.unwrap(), channel.unwrap(), mode.unwrap());
    }
}

pub async fn call_set_gain_mode(inputs:&[u8], mut dac_manager:DacManager<'static>) {
    let input_dac_id  = inputs[1]; // usize
    let input_channel = inputs[2]; // mcp4728::Channel
    let input_gain_mode = inputs[3]; // mcp4728::GainMode

    let dac_id = match_dac_id(input_dac_id).await;
    let channel = match_channel(input_channel).await;
    let mode = match_gain_mode(input_gain_mode).await;

    if dac_id.is_some() && channel.is_some() && mode.is_some() {
        dac_manager.set_gain_mode(dac_id.unwrap(), channel.unwrap(), mode.unwrap());
    }
}

pub async fn call_set_vref_mode(inputs:&[u8], mut dac_manager:DacManager<'static>) {
    let input_dac_id  = inputs[1]; // usize
    let input_channel = inputs[2]; // mcp4728::Channel
    let input_vref_mode = inputs[3]; // mcp4728::VoltageReferenceMode

    let dac_id = match_dac_id(input_dac_id).await;
    let channel = match_channel(input_channel).await;
    let mode = match_vref_mode(input_vref_mode).await;

    if dac_id.is_some() && channel.is_some() && mode.is_some() {
        dac_manager.set_vref_mode(dac_id.unwrap(), channel.unwrap(), mode.unwrap());
    }
}

pub async fn call_set_voltage(inputs:&[u8], mut dac_manager:DacManager<'static>) {
    let input_dac_id  = inputs[1]; // usize
    let input_channel = inputs[2]; // Channel
    let input_voltage_higher_order = inputs[3]; // part of u16
    let input_voltage_lower_order  = inputs[4]; // part of u16
    let input_voltage = two_u8_into_u16(
        input_voltage_higher_order,
         input_voltage_lower_order).await; // u16

    let dac_id = match_dac_id(input_dac_id).await;
    let channel = match_channel(input_channel).await;
    let voltage = match_voltage(input_voltage).await;

    if dac_id.is_some() && channel.is_some() && voltage.is_some() {
        dac_manager.set_voltage(dac_id.unwrap(), channel.unwrap(), voltage.unwrap());
    }
}

async fn match_dac_id(input_dac_id:u8) -> Option<usize>{
    match input_dac_id {
        0 => {None},
        10 => {Some(0)},
        20 => {Some(1)},
        30 => {Some(2)},
        40 => {Some(3)},
        50 => {Some(4)},
        60 => {Some(5)},
        _ => {None},
    }
}

async fn match_channel(input_channel:u8) -> Option<mcp4728::Channel> {
    match input_channel {
        0 => {None},
        11 => {Some(mcp4728::Channel::A)},
        21 => {Some(mcp4728::Channel::B)},
        31 => {Some(mcp4728::Channel::C)},
        41 => {Some(mcp4728::Channel::D)},
        _ => {None},
    }
}

async fn match_voltage(input_voltage:u16) -> Option<u16> {
    match input_voltage {
        0 => {None},
        val => {Some(val)},
    }
}

async fn match_power_down_mode(input_power_down_mode:u8) -> Option<mcp4728::PowerDownMode> {
    match input_power_down_mode {
        0 => {None},
        14 => {Some(mcp4728::PowerDownMode::Normal)},
        24 => {Some(mcp4728::PowerDownMode::PowerDownOneK)},
        34 => {Some(mcp4728::PowerDownMode::PowerDownOneHundredK)},
        44 => {Some(mcp4728::PowerDownMode::PowerDownFiveHundredK)},
        _ => {None},
    }
}

async fn match_gain_mode(input_gain_mode:u8) -> Option<mcp4728::GainMode> {
    match input_gain_mode {
        0 => {None},
        13 => {Some(mcp4728::GainMode::TimesOne)},
        23 => {Some(mcp4728::GainMode::TimesTwo)},
        _ => {None},
    }
}

async fn match_vref_mode(input_vref_mode:u8) -> Option<mcp4728::VoltageReferenceMode> {
    match input_vref_mode {
        0 => {None},
        12 => {Some(mcp4728::VoltageReferenceMode::External)},
        22 => {Some(mcp4728::VoltageReferenceMode::Internal)},
        _ => {None},
    }
}