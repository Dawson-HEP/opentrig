use defmt::*;

use crate::dac;
use crate::dac::DacManager;

/// functions and their inputs
/// NOTE : inputs of 0 are reserved for errors
/// NOTE : input 0 is the first input after 0xFF, which is the start value of the input
/// fn                       |  input_0(function_id) |  input_1  |  input_2  |  input_3     |  input_4
/// set_voltage              |         1             |   dac_id  |  channel  | voltage u8_1 | voltage u8_2
/// set_vref_mode            |         2             |   dac_id  |  channel  |    mode      |    N/A
/// set_gain_mode            |         3             |   dac_id  |  channel  |    mode      |    N/A
/// set_power_down_mode      |         4             |   dac_id  |  channel  |    mode      |    N/A
/// set_all_voltages         |         5             |   is_all  |    (2 or 48 inputs : u16-composants)
/// set_all_vref_modes       |         6             |   is_all  |    (1 or 24 inputs :  u8-composants)
/// set_all_gain_modes       |         7             |   is_all  |    (1 or 24 inputs :  u8-composants)
/// set_all_power_down_modes |         8             |   is_all  |    (1 or 24 inputs :  u8-composants)
/// set_voltage_by_dac_id    |     for the moment, not going to be implemented
/// set_vref_mode_by_dac_id  |     for the moment, not going to be implemented
/// set_gain_mode_by_dac_id  |     for the moment, not going to be implemented
/// set_power_down_by_dac_id |     for the moment, not going to be implemented
/// others?

pub async fn match_cli_values_to_functions(inputs: &[u8], dac_manager: &mut DacManager<'static>) {
    println!("function_id is {:?}", inputs[0]);

    match inputs[0] {
        0 => {
            info!("error in input, function 0 is undefined")
        }
        1 => call_set_voltage(inputs, dac_manager).await,
        2 => call_set_vref_mode(inputs, dac_manager).await,
        3 => call_set_gain_mode(inputs, dac_manager).await,
        4 => call_set_power_down_mode(inputs, dac_manager).await,
        5 => call_set_all_voltages(inputs, dac_manager).await,
        6 => call_set_all_vref_modes(inputs, dac_manager).await,
        7 => call_set_all_gain_modes(inputs, dac_manager).await,
        8 => call_set_all_power_down_modes(inputs, dac_manager).await,
        f => {
            println!("error in input, function {} is undefined", f)
        }
    }
}

//
//
//
//
//
//
//
//
//
//
//
//
//
//     PRIVATE FUNCTIONS
//
//
//
//
//
//
//
//
//
//
//
//

enum fn_call {
    Voltage,
    Vref_mode,
    Gain_mode,
    PowerDown_mode,
}

enum setting_values {
    Vref([mcp4728::VoltageReferenceMode; 24]),
    Gain([mcp4728::GainMode; 24]),
    PowerDown([mcp4728::PowerDownMode; 24]),
}

async fn two_u8_into_u16(u8_higher_order: u8, u8_lower_order: u8) -> u16 {
    ((u8_higher_order as u16) << 8) | u8_lower_order as u16
}

async fn call_set_all_power_down_modes(inputs: &[u8], mut dac_manager: &mut DacManager<'static>) {
    let input_set_all = inputs[1];
    let set_all = match_set_all(input_set_all).await;

    match set_all {
        Some(true) => {
            let input_power_down_mode = inputs[2];
            let power_down_mode = match_power_down_mode(input_power_down_mode);

            if power_down_mode.is_some() {
                info!("valid call to set_all_power_down_modes");
                dac_manager
                    .set_all_power_down_modes([power_down_mode.unwrap(); 24])
                    .await
                    .unwrap();
            } else {
                info!("invalid call to set_all_power_down_modes casting one value to 24")
            }
        }
        Some(false) => {
            let power_down_mode_option =
                from_idx_get_valid_type(2, inputs, fn_call::PowerDown_mode).await;
            if power_down_mode_option.is_some() {
                if let setting_values::PowerDown(power_down_modes) = power_down_mode_option.unwrap()
                {
                    info!("valid call to set_all_power_down_modes");
                    dac_manager
                        .set_all_power_down_modes(power_down_modes)
                        .await
                        .unwrap();
                } else {
                    info!("invalid call to set_all_power_down_modes 24 individual values")
                }
            }
        }
        None => {
            info!("invalid call to set_all_power_down_modes")
        }
    }
}

async fn call_set_all_gain_modes(inputs: &[u8], mut dac_manager: &mut DacManager<'static>) {
    let input_set_all = inputs[1];
    let set_all = match_set_all(input_set_all).await;

    match set_all {
        Some(true) => {
            let input_gain_mode = inputs[2];
            let gain_mode = match_gain_mode(input_gain_mode);

            if gain_mode.is_some() {
                info!("valid call to set_all_gain_modes");
                dac_manager
                    .set_all_gain_modes([gain_mode.unwrap(); 24])
                    .await
                    .unwrap();
            } else {
                info!("invalid call to set_all_gain_modes casting one value to 24")
            }
        }
        Some(false) => {
            let gain_mode_option = from_idx_get_valid_type(2, inputs, fn_call::Gain_mode).await;
            if gain_mode_option.is_some() {
                if let setting_values::Gain(gain_modes) = gain_mode_option.unwrap() {
                    info!("valid call to set_all_gain_modes");
                    dac_manager.set_all_gain_modes(gain_modes).await.unwrap();
                } else {
                    info!("invalid call to set_all_gain_modes 24 individual values")
                }
            }
        }
        None => {
            info!("invalid call to set_all_gain_modes")
        }
    }
}

async fn call_set_all_vref_modes(inputs: &[u8], mut dac_manager: &mut DacManager<'static>) {
    let input_set_all = inputs[1];
    let set_all = match_set_all(input_set_all).await;

    match set_all {
        Some(true) => {
            let input_vref_mode = inputs[2];
            let vref = match_vref_mode(input_vref_mode);

            if vref.is_some() {
                info!("valid call to set_all_vref_modes");
                dac_manager
                    .set_all_vref_modes([vref.unwrap(); 24])
                    .await
                    .unwrap();
            } else {
                info!("invalid call to set_all_vref_modes casting one value to 24")
            }
        }
        Some(false) => {
            let vref_option = from_idx_get_valid_type(2, inputs, fn_call::Vref_mode).await;
            if vref_option.is_some() {
                if let setting_values::Vref(vref_modes) = vref_option.unwrap() {
                    info!("valid call to set_all_vref_modes");
                    dac_manager.set_all_vref_modes(vref_modes).await.unwrap();
                } else {
                    info!("invalid call to set_all_vref_modes 24 individual values")
                }
            }
        }
        None => {
            info!("invalid call to set_all_vref_modes")
        }
    }
}

async fn call_set_all_voltages(inputs: &[u8], mut dac_manager: &mut DacManager<'static>) {
    let input_set_all = inputs[1];
    let set_all = match_set_all(input_set_all).await;

    match set_all {
        Some(true) => {
            let input_voltage_higher_order = inputs[2]; // part of u16
            let input_voltage_lower_order = inputs[3]; // part of u16
            let input_voltage =
                two_u8_into_u16(input_voltage_higher_order, input_voltage_lower_order).await; // u16

            println!("{}", input_voltage);

            let voltage = match_voltage(input_voltage).await;
            if voltage.is_some() {
                info!("valid call to set_all_voltages");
                dac_manager
                    .set_all_voltages([voltage.unwrap(); 24])
                    .await
                    .unwrap();
            } else {
                info!("invalid call to set_all_voltages casting one to 24")
            }
        }
        Some(false) => {
            let voltages = from_idx_get_valid_voltages(2, inputs).await;
            if voltages.is_some() {
                info!("valid call to set_all_voltages");
                dac_manager
                    .set_all_voltages(voltages.unwrap())
                    .await
                    .unwrap();
            } else {
                info!("invalid call to set_all_voltages 24 individual values")
            }
        }
        None => {
            info!("invalid call to set_all_voltages")
        }
    }
}

async fn call_set_power_down_mode(inputs: &[u8], mut dac_manager: &mut DacManager<'static>) {
    let input_dac_id = inputs[1]; // usize
    let input_channel = inputs[2]; // mcp4728::Channel
    let input_power_down_mode = inputs[3]; // mcp4728::PowerDownMode

    let dac_id = match_dac_id(input_dac_id).await;
    let channel = match_channel(input_channel).await;
    let mode = match_power_down_mode(input_power_down_mode);

    if dac_id.is_some() && channel.is_some() && mode.is_some() {
        info!("valid call to set_power_down_mode");
        dac_manager
            .set_power_down_mode(dac_id.unwrap(), channel.unwrap(), mode.unwrap())
            .await
            .unwrap();
    } else {
        info!("invalid call to set_power_down_mode")
    }
}

async fn call_set_gain_mode(inputs: &[u8], mut dac_manager: &mut DacManager<'static>) {
    let input_dac_id = inputs[1]; // usize
    let input_channel = inputs[2]; // mcp4728::Channel
    let input_gain_mode = inputs[3]; // mcp4728::GainMode

    let dac_id = match_dac_id(input_dac_id).await;
    let channel = match_channel(input_channel).await;
    let mode = match_gain_mode(input_gain_mode);

    if dac_id.is_some() && channel.is_some() && mode.is_some() {
        info!("valid call to set_gain_mode");
        dac_manager
            .set_gain_mode(dac_id.unwrap(), channel.unwrap(), mode.unwrap())
            .await
            .unwrap();
    } else {
        info!("invalid call to set_gain_mode")
    }
}

async fn call_set_vref_mode(inputs: &[u8], mut dac_manager: &mut DacManager<'static>) {
    let input_dac_id = inputs[1]; // usize
    let input_channel = inputs[2]; // mcp4728::Channel
    let input_vref_mode = inputs[3]; // mcp4728::VoltageReferenceMode

    let dac_id = match_dac_id(input_dac_id).await;
    let channel = match_channel(input_channel).await;
    let mode = match_vref_mode(input_vref_mode);

    if dac_id.is_some() && channel.is_some() && mode.is_some() {
        info!("valid call to set_vref_mode");
        dac_manager
            .set_vref_mode(dac_id.unwrap(), channel.unwrap(), mode.unwrap())
            .await
            .unwrap();
    } else {
        info!("invalid call to set_vref_mode")
    }
}

async fn call_set_voltage(inputs: &[u8], mut dac_manager: &mut DacManager<'static>) {
    let input_dac_id = inputs[1]; // usize
    let input_channel = inputs[2]; // Channel
    let input_voltage_higher_order = inputs[3]; // part of u16
    let input_voltage_lower_order = inputs[4]; // part of u16
    let input_voltage =
        two_u8_into_u16(input_voltage_higher_order, input_voltage_lower_order).await; // u16

    let dac_id = match_dac_id(input_dac_id).await;
    let channel = match_channel(input_channel).await;
    let voltage = match_voltage(input_voltage).await;

    if dac_id.is_some() && channel.is_some() && voltage.is_some() {
        info!("valid call to set_voltage");
        dac_manager
            .set_voltage(dac_id.unwrap(), channel.unwrap(), voltage.unwrap())
            .await
            .unwrap();
    } else {
        info!("invalid call to set_voltage")
    }
}

async fn match_dac_id(input_dac_id: u8) -> Option<usize> {
    match input_dac_id {
        0 => {
            info!("invalid dac_id present");
            None
        }
        10 => Some(0),
        20 => Some(1),
        30 => Some(2),
        40 => Some(3),
        50 => Some(4),
        60 => Some(5),
        _ => {
            info!("invalid dac_id present");
            None
        }
    }
}

async fn match_channel(input_channel: u8) -> Option<mcp4728::Channel> {
    match input_channel {
        0 => {
            info!("invalid channel present");
            None
        }
        11 => Some(mcp4728::Channel::A),
        21 => Some(mcp4728::Channel::B),
        31 => Some(mcp4728::Channel::C),
        41 => Some(mcp4728::Channel::D),
        _ => {
            info!("invalid channel present");
            None
        }
    }
}

async fn match_voltage(input_voltage: u16) -> Option<u16> {
    match input_voltage {
        0 => {
            info!("invalid voltage present");
            None
        }
        val => Some(val),
    }
}

fn match_power_down_mode(input_power_down_mode: u8) -> Option<mcp4728::PowerDownMode> {
    match input_power_down_mode {
        0 => {
            info!("invalid power_down_mode present");
            None
        }
        14 => Some(mcp4728::PowerDownMode::Normal),
        24 => Some(mcp4728::PowerDownMode::PowerDownOneK),
        34 => Some(mcp4728::PowerDownMode::PowerDownOneHundredK),
        44 => Some(mcp4728::PowerDownMode::PowerDownFiveHundredK),
        _ => {
            info!("invalid power_down_mode present");
            None
        }
    }
}

fn match_gain_mode(input_gain_mode: u8) -> Option<mcp4728::GainMode> {
    match input_gain_mode {
        0 => {
            info!("invalid gain_mode present");
            None
        }
        13 => Some(mcp4728::GainMode::TimesOne),
        23 => Some(mcp4728::GainMode::TimesTwo),
        _ => {
            info!("invalid gain_mode present");
            None
        }
    }
}

async fn match_set_all(input_set_all: u8) -> Option<bool> {
    match input_set_all {
        0 => {
            info!("invalid value for set_all present");
            None
        }
        100 => Some(true),
        200 => Some(false),
        _ => {
            info!("invalid value for set_all present");
            None
        }
    }
}

fn match_vref_mode(input_vref_mode: u8) -> Option<mcp4728::VoltageReferenceMode> {
    match input_vref_mode {
        0 => {
            info!("invalid vref_mode present");
            None
        }
        12 => Some(mcp4728::VoltageReferenceMode::External),
        22 => Some(mcp4728::VoltageReferenceMode::Internal),
        _ => {
            info!("invalid vref_mode present");
            None
        }
    }
}

async fn from_idx_get_valid_type(
    idx: usize,
    inputs: &[u8],
    set_type: fn_call,
) -> Option<setting_values> {
    let u8_1 = inputs[idx];
    let u8_2 = inputs[idx + 1];
    let u8_3 = inputs[idx + 2];
    let u8_4 = inputs[idx + 3];
    let u8_5 = inputs[idx + 4];
    let u8_6 = inputs[idx + 5];
    let u8_7 = inputs[idx + 6];
    let u8_8 = inputs[idx + 7];
    let u8_9 = inputs[idx + 8];
    let u8_10 = inputs[idx + 9];
    let u8_11 = inputs[idx + 10];
    let u8_12 = inputs[idx + 11];
    let u8_13 = inputs[idx + 12];
    let u8_14 = inputs[idx + 13];
    let u8_15 = inputs[idx + 14];
    let u8_16 = inputs[idx + 15];
    let u8_17 = inputs[idx + 16];
    let u8_18 = inputs[idx + 17];
    let u8_19 = inputs[idx + 18];
    let u8_20 = inputs[idx + 19];
    let u8_21 = inputs[idx + 20];
    let u8_22 = inputs[idx + 21];
    let u8_23 = inputs[idx + 22];
    let u8_24 = inputs[idx + 23];

    let u8_items = [
        u8_1, u8_2, u8_3, u8_4, u8_5, u8_6, u8_7, u8_8, u8_9, u8_10, u8_11, u8_12, u8_13, u8_14,
        u8_15, u8_16, u8_17, u8_18, u8_19, u8_20, u8_21, u8_22, u8_23, u8_24,
    ];

    match set_type {
        fn_call::Vref_mode => {
            let mapped_vref = u8_items.map(|vref| match_vref_mode(vref).unwrap());
            let vref_validity = u8_items.map(|vref| match_vref_mode(vref).is_some());
            let num_true = vref_validity.iter().count();
            if num_true == 24 {
                Some(setting_values::Vref(mapped_vref))
            } else {
                info!("not 24 correct vref modes");
                None
            }
        }
        fn_call::Gain_mode => {
            let mapped_gain_mode = u8_items.map(|gain_mode| match_gain_mode(gain_mode).unwrap());
            let gain_mode_validity = u8_items.map(|gain_mode| match_gain_mode(gain_mode).is_some());
            let num_true = gain_mode_validity.iter().count();
            if num_true == 24 {
                Some(setting_values::Gain(mapped_gain_mode))
            } else {
                info!("not 24 correct gain modes");
                None
            }
        }
        fn_call::PowerDown_mode => {
            let mapped_power_down =
                u8_items.map(|power_down| match_power_down_mode(power_down).unwrap());
            let power_down_validity =
                u8_items.map(|power_down| match_power_down_mode(power_down).is_some());
            let num_true = power_down_validity.iter().count();
            if num_true == 24 {
                Some(setting_values::PowerDown(mapped_power_down))
            } else {
                info!("not 24 correct power_down modes");
                None
            }
        }
        fn_call::Voltage => {
            Err::<bool, &str>("not implemented yet").unwrap();
            None
        }
    }
}

async fn from_idx_get_valid_voltages(idx: usize, inputs: &[u8]) -> Option<[u16; 24]> {
    let u8_1 = inputs[idx];
    let u8_2 = inputs[idx + 1];
    let u8_3 = inputs[idx + 2];
    let u8_4 = inputs[idx + 3];
    let u8_5 = inputs[idx + 4];
    let u8_6 = inputs[idx + 5];
    let u8_7 = inputs[idx + 6];
    let u8_8 = inputs[idx + 7];
    let u8_9 = inputs[idx + 8];
    let u8_10 = inputs[idx + 9];
    let u8_11 = inputs[idx + 10];
    let u8_12 = inputs[idx + 11];
    let u8_13 = inputs[idx + 12];
    let u8_14 = inputs[idx + 13];
    let u8_15 = inputs[idx + 14];
    let u8_16 = inputs[idx + 15];
    let u8_17 = inputs[idx + 16];
    let u8_18 = inputs[idx + 17];
    let u8_19 = inputs[idx + 18];
    let u8_20 = inputs[idx + 19];
    let u8_21 = inputs[idx + 20];
    let u8_22 = inputs[idx + 21];
    let u8_23 = inputs[idx + 22];
    let u8_24 = inputs[idx + 23];
    let u8_25 = inputs[idx + 24];
    let u8_26 = inputs[idx + 25];
    let u8_27 = inputs[idx + 26];
    let u8_28 = inputs[idx + 27];
    let u8_29 = inputs[idx + 28];
    let u8_30 = inputs[idx + 29];
    let u8_31 = inputs[idx + 30];
    let u8_32 = inputs[idx + 31];
    let u8_33 = inputs[idx + 32];
    let u8_34 = inputs[idx + 33];
    let u8_35 = inputs[idx + 34];
    let u8_36 = inputs[idx + 35];
    let u8_37 = inputs[idx + 36];
    let u8_38 = inputs[idx + 37];
    let u8_39 = inputs[idx + 38];
    let u8_40 = inputs[idx + 39];
    let u8_41 = inputs[idx + 40];
    let u8_42 = inputs[idx + 41];
    let u8_43 = inputs[idx + 42];
    let u8_44 = inputs[idx + 43];
    let u8_45 = inputs[idx + 44];
    let u8_46 = inputs[idx + 45];
    let u8_47 = inputs[idx + 46];
    let u8_48 = inputs[idx + 47];

    let volt_1 = match_voltage(two_u8_into_u16(u8_1, u8_2).await).await;
    let volt_2 = match_voltage(two_u8_into_u16(u8_3, u8_4).await).await;
    let volt_3 = match_voltage(two_u8_into_u16(u8_5, u8_6).await).await;
    let volt_4 = match_voltage(two_u8_into_u16(u8_7, u8_8).await).await;
    let volt_5 = match_voltage(two_u8_into_u16(u8_9, u8_10).await).await;
    let volt_6 = match_voltage(two_u8_into_u16(u8_11, u8_12).await).await;
    let volt_7 = match_voltage(two_u8_into_u16(u8_13, u8_14).await).await;
    let volt_8 = match_voltage(two_u8_into_u16(u8_15, u8_16).await).await;
    let volt_9 = match_voltage(two_u8_into_u16(u8_17, u8_18).await).await;
    let volt_10 = match_voltage(two_u8_into_u16(u8_19, u8_20).await).await;
    let volt_11 = match_voltage(two_u8_into_u16(u8_21, u8_22).await).await;
    let volt_12 = match_voltage(two_u8_into_u16(u8_23, u8_24).await).await;
    let volt_13 = match_voltage(two_u8_into_u16(u8_25, u8_26).await).await;
    let volt_14 = match_voltage(two_u8_into_u16(u8_27, u8_28).await).await;
    let volt_15 = match_voltage(two_u8_into_u16(u8_29, u8_30).await).await;
    let volt_16 = match_voltage(two_u8_into_u16(u8_31, u8_32).await).await;
    let volt_17 = match_voltage(two_u8_into_u16(u8_33, u8_34).await).await;
    let volt_18 = match_voltage(two_u8_into_u16(u8_35, u8_36).await).await;
    let volt_19 = match_voltage(two_u8_into_u16(u8_37, u8_38).await).await;
    let volt_20 = match_voltage(two_u8_into_u16(u8_39, u8_40).await).await;
    let volt_21 = match_voltage(two_u8_into_u16(u8_41, u8_42).await).await;
    let volt_22 = match_voltage(two_u8_into_u16(u8_43, u8_44).await).await;
    let volt_23 = match_voltage(two_u8_into_u16(u8_45, u8_46).await).await;
    let volt_24 = match_voltage(two_u8_into_u16(u8_47, u8_48).await).await;

    if volt_1.is_some()
        && volt_2.is_some()
        && volt_3.is_some()
        && volt_4.is_some()
        && volt_5.is_some()
        && volt_6.is_some()
        && volt_7.is_some()
        && volt_8.is_some()
        && volt_9.is_some()
        && volt_10.is_some()
        && volt_11.is_some()
        && volt_12.is_some()
        && volt_13.is_some()
        && volt_14.is_some()
        && volt_15.is_some()
        && volt_16.is_some()
        && volt_17.is_some()
        && volt_18.is_some()
        && volt_19.is_some()
        && volt_20.is_some()
        && volt_21.is_some()
        && volt_22.is_some()
        && volt_23.is_some()
        && volt_24.is_some()
    {
        Some([
            volt_1.unwrap(),
            volt_2.unwrap(),
            volt_3.unwrap(),
            volt_4.unwrap(),
            volt_5.unwrap(),
            volt_6.unwrap(),
            volt_7.unwrap(),
            volt_8.unwrap(),
            volt_9.unwrap(),
            volt_10.unwrap(),
            volt_11.unwrap(),
            volt_12.unwrap(),
            volt_13.unwrap(),
            volt_14.unwrap(),
            volt_15.unwrap(),
            volt_16.unwrap(),
            volt_17.unwrap(),
            volt_18.unwrap(),
            volt_19.unwrap(),
            volt_20.unwrap(),
            volt_21.unwrap(),
            volt_22.unwrap(),
            volt_23.unwrap(),
            volt_24.unwrap(),
        ])
    } else {
        info!("not 24 correct u16 valid voltages");
        None
    }
}
