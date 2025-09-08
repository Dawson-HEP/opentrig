use clap::Parser;

mod rusb_communication;

/// CLI for pico
#[derive(Parser, Default, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// message to send and receive from pico
    #[arg(short, long, required=false)]
    message: Option<String>,
}

fn main() {
    let args = Args::parse();
    
    match args.message {
        Some(msg) => {
            rusb_communication::write_bulk(msg.as_str().as_bytes());
            rusb_communication::read_bulk();
        },
        _ => {},
    }
    
}