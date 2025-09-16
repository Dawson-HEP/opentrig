use futures_lite::future::block_on;
use nusb::transfer::RequestBuffer;
use std::fs::{OpenOptions};
use std::io::{Write};

// const BULK_OUT_EP: u8 = 0x01;
const BULK_IN_EP: u8 = 0x81;


fn main() {
    let di = nusb::list_devices()
        .unwrap()
        .find(|d| d.vendor_id() == 0xc0de && d.product_id() == 0xcafe)
        .expect("no device found");

    let device = di.open().expect("error opening device");
    let interface = device.claim_interface(0).expect("error claiming interface");
    // let result = block_on(interface.bulk_out(BULK_OUT_EP, b"hello world".into()));
    // println!("{result:?}");

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("output.csv").unwrap();

    loop {
        let result = block_on(interface.bulk_in(BULK_IN_EP, RequestBuffer::new(16)));
        let raw_data: [u8; 16] = result.data.try_into().unwrap();

        let (start_byte, end_byte) = (raw_data[0], raw_data[15]);
        if start_byte != 0x7E {
            panic!("start byte error");
        }
        if end_byte != 0x7D {
            panic!("start byte error");
        }

        let trigger_id_buf = &raw_data[1..3];
        let trigger_clk_buf = &raw_data[3..11];
        let trigger_data_buf = &raw_data[11..15];

        let trigger_id = u16::from_be_bytes(trigger_id_buf.try_into().unwrap());
        let trigger_clk = u64::from_be_bytes(trigger_clk_buf.try_into().unwrap());
        let data_clk_buf = u32::from_be_bytes(trigger_data_buf.try_into().unwrap());
        let trigger_data = data_clk_buf & 0x00FF_FFFF;
        let veto_in = (data_clk_buf >> 31 & 1) != 0;
        let internal_trigger = (data_clk_buf >> 30 & 1) != 0;

        let d = trigger_data;
    
        let line = format!(
            "{},{},{},{:08b},{:08b},{:08b},{:08b},{},{}\n",
            trigger_id,
            trigger_clk,
            d,
            (d >> 24) & 0xFF,
            (d >> 16) & 0xFF,
            (d >> 8) & 0xFF,
            d & 0xFF,
            veto_in,
            internal_trigger,
        );

        println!("{}", line);
        file.write_all(line.as_bytes()).unwrap();
    }
}
