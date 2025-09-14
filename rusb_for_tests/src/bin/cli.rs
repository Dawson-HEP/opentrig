use clap::Parser;

mod rusb_stuff;


/// CLI for pico
#[derive(Parser, Default, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// function to callon pico
    #[arg(short, long, required=false)]
    function: Option<&str>,

    /// dac-id to modify
    #[arg(short, long, required=false)]
    dac_id: Option<u8>,

    /// dac channel to modify
    #[arg(short, long, required=false)]
    channel: Option<&str>,

    /// voltage value for modification
    #[arg(short, long, required=false)]
    voltage: Option<u16>,

    /// vref-mode for modification
    #[arg(short, long, required=false)]
    vref_mode: Option<&str>,

    /// gain-mode for modification
    #[arg(short, long, required=false)]
    gain_mode: Option<&str>,

    /// powerdown-down mode to modify
    #[arg(short, long, required=false)]
    powerdown_mode: Option<&str>,

    /// set-all dac id's and channels uniformly or uniquely
    #[arg(short, long, required=false)]
    setall: Option<bool>,
}

fn main() {
    let args = Args::parse();

    let mut data : Vec<u8> = vec!(0xFF);
    
    match args.function {
        Some("set_voltage") 
        Some("set_vref_mode")
        Some("set_voltage")
        Some("set_voltage")
        Some("set_voltage")
        Some("set_voltage")
        Some("set_voltage")
        Some("set_voltage")
        Some(msg) => {
            data.push(msg);
            //rusb_communication::write_bulk(msg.as_str().as_bytes());
            //rusb_communication::read_bulk();
        },
        _ => {},
    }    
    rusb_stuff::rusb_demo(&data);
}