use rusb;
use rusb::DeviceHandle;
use rusb::UsbContext;
use rusb::Device;
use rusb::Context;


use serde::{Serialize, Deserialize};
use postcard::from_bytes;



const PICO_VID : u16 = 49374;
const PICO_PID : u16 = 51966;
//const IFACE_0_END_OUT : u8 = 1;
//const IFACE_0_END_IN : u8 = 129;
const IFACE_0_END_OUT : u8 = 1;
const IFACE_0_END_IN : u8 = 129;

//const PICO_VID : u16 = 11914;
//const PICO_PID : u16 = 12;
//
//const IFACE_0_END_OUT : u8 = 4;
//const IFACE_0_END_IN : u8 = 133;
//
//const IFACE_1_END_IN : u8 = 129;
//
//const IFACE_2_END_OUT : u8 = 2;
//const IFACE_2_END_IN : u8 = 131;


fn _display_device_info(device : &Device<Context>) {
    let device_desc = device.device_descriptor().unwrap();
    println!("{:?}", device_desc);

    println!("");

    for n in 0..device_desc.num_configurations() {
        let config_desc = device.config_descriptor(n).unwrap();
        println!("{:?}", config_desc);

        println!("");

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                println!("{:?}", interface_desc);
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    println!("{:?}", endpoint_desc);
                    println!("{:?}", endpoint_desc.direction());
                }
                println!("");
            }
        }
    }

}


fn get_pico_prepared_for_iface() -> DeviceHandle<Context> {
    let context = rusb::Context::new().expect("failed to get rusb context");
    let pico_handle = context
            .open_device_with_vid_pid(PICO_VID, PICO_PID)
            .expect("failed to open pico with VID and PID");

    //println!("{:?}", pico_handle.release_interface(0));
    //println!("{:?}", pico_handle.release_interface(1));
    //println!("{:?}", pico_handle.release_interface(2));
    pico_handle.claim_interface(0)
            .expect("failed to claim communication interface");
    //pico_handle.claim_interface(1)
    //        .expect("failed to claim communication interface");
    //pico_handle.claim_interface(2)
    //        .expect("failed to claim communication interface");

    
    //println!("");
    //println!("{:?}", pico_handle.release_interface(0));
    //println!("{:?}", pico_handle.release_interface(1));
    //println!("{:?}", pico_handle.release_interface(2));

    pico_handle
}


fn _get_pico_device(pico_handle : &DeviceHandle<Context>) -> Device<Context> {
    pico_handle.device()
}

fn _list_devices() {
    for device in rusb::devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();
        let device_handle = device.open();
    
        println!("name {:?} Bus {:03} Device {:03} ID {:}:{:} {:?}",
            device_desc.manufacturer_string_index(),
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id(),
            device_handle,);
    }
}


pub fn write_bulk(data:&[u8], pico_handle : &DeviceHandle<Context>) {

    //let time_write = std::time::Duration::from_millis(1);
    let time_write = std::time::Duration::from_secs(1);

    //let a = pico_handle.read_bulk(end, buffer, time);
    let write_result = pico_handle.write_bulk(IFACE_0_END_OUT, data, time_write);
    match write_result {
        Ok(_) => println!("sent data {:?}", data),
        //Ok(_) => println!("sent data {:?}", str::from_utf8(data).unwrap()),
        Err(n) => println!("didn't write {:?}", n),
    }
}


pub fn read_bulk(pico_handle : &DeviceHandle<Context>) {

    let time_read = std::time::Duration::from_secs(3);
        
    let mut read_buf: [u8; 4096] = [0; 4096];
    
    let read_result = pico_handle.read_bulk(IFACE_0_END_IN, &mut read_buf, time_read);
    
    match read_result {
        Ok(_) => {
            println!("received data");
            let restruct = from_bytes::<DAQSample>(&read_buf).unwrap();
            println!("{:?}", restruct);
            //let formatted_received = str::from_utf8(&read_buf).unwrap();
            //let cleaned = formatted_received.replace("\0", "");
            //println!("received data, formatted to : {:?}", cleaned);
        },
        Err(n) => println!("didn't read {:?}", n),
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct DAQSample {
    pub trigger_id: u16,
    pub trigger_clk: u64,
    pub trigger_data: u32,
    pub veto_in: bool,
    pub internal_trigger: bool,
}

pub fn rusb_demo(data:&[u8]) {

    let pico_handle = get_pico_prepared_for_iface();
    
    //_display_device_info(&get_pico_device(&pico_handle));

    //let data = "pico sent and received!!!".as_bytes();
    //let data = "pico!!!".as_bytes();
    //let data: &[u8] = &[0xFF, 1, 2, 3, 4, 5];
    //let data: &[u8] = &[0xff, 1, 20, 31, 0, 255, 6, 7];
    //let data: &[u8] = &[0xff, 1, 2, 3, 4, 5, 6, 7];
    //let data: &[u8] = &[0xff, 5, 100, 2, 12];
    
    
    //
    //
    //
    // REMINDER
    // ADD START/END CHARACTER U8
    // THIS WAY FIXES FILTERING
    // REMINDER
    //
    //
    
    
    
    
    write_bulk(data, &pico_handle);
    read_bulk(&pico_handle);
}


fn main() {
    _list_devices();
    rusb_demo("a".as_bytes());
}