use clap::Parser;

mod rusb_stuff;


/// CLI for pico
#[derive(Parser, Default, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// message to send and receive from pico
    #[arg(short, long, required=false)]
    function: Option<u8>,
    /// message to send and receive from pico
    #[arg(short, long, required=false)]
    setall: Option<u8>,
    /// message to send and receive from pico
    #[arg(long, required=false)]
    vmsb: Option<u8>,
    /// message to send and receive from pico
    #[arg(long, required=false)]
    vlsb: Option<u8>,


}

fn main() {
    let args = Args::parse();

    let mut data : Vec<u8> = vec!(0xFF);
    
    match args.function {
        Some(msg) => {
            data.push(msg);
            //rusb_communication::write_bulk(msg.as_str().as_bytes());
            //rusb_communication::read_bulk();
        },
        _ => {},
    }
    
    match args.setall {
        Some(msg) => {
            data.push(msg);
            //rusb_communication::write_bulk(msg.as_str().as_bytes());
            //rusb_communication::read_bulk();
        },
        _ => {},
    }
    
    match args.vmsb {
        Some(msg) => {
            data.push(msg);
            //rusb_communication::write_bulk(msg.as_str().as_bytes());
            //rusb_communication::read_bulk();
        },
        _ => {},
    }
    
    match args.vlsb {
        Some(msg) => {
            data.push(msg);
            //rusb_communication::write_bulk(msg.as_str().as_bytes());
            //rusb_communication::read_bulk();
        },
        _ => {},
    }
    
    rusb_stuff::rusb_demo(&data);
}