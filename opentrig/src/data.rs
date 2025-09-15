use core::ops::Deref;
use defmt::Format;
use heapless::Vec;

use defmt;

pub struct DAQSample {
    pub trigger_id: u16,
    pub trigger_clk: u64,
    pub trigger_data: u32,
    pub veto_in: bool,
    pub internal_trigger: bool,
}


// [
//  u8 for start (0xFF),
//  2 u8's for trigger_id (glue them together left to right),
//  8 u8's for trigger_clk (glue them together left to right),
//  4 u8's for trigger_data (glue them together from left to right),
//  u8 for veto_in, internal_trigger, end_confirmation (
//  msb is veto_in, 2nd_msb is internal_trigger, 6 lsb is end confirmation 0x3F
//  )
//  ]


pub fn u64_to_bytes(u:u64) -> [u8; 8] {
    let nbits = 64;
    let mut bytes: [u8; 8] = [0;8];
    for i in 0..8 {
        bytes[i] = (((u <<  i*8) >> nbits-8) as u8);
    };
    bytes
}
pub fn u32_to_bytes(u:u32) -> [u8; 4] {
    let nbits = 32;
    let mut bytes: [u8; 4] = [0;4];
    for i in 0..4 {
        bytes[i] = (((u <<  i*8) >> nbits-8) as u8);
    };
    bytes
}
pub fn u16_to_bytes(u:u16) -> [u8; 2] {
    let nbits = 16;
    let mut bytes: [u8; 2] = [0;2];
    for i in 0..2 {
        bytes[i] = (((u <<  i*8) >> nbits-8) as u8);
    };
    bytes
}

fn concat_arrays<T, const A: usize, const B: usize, const C: usize>(a: [T; A], b: [T; B]) -> [T; C] {
    assert_eq!(A+B, C);
    let mut iter = a.into_iter().chain(b);
    core::array::from_fn(|_| iter.next().unwrap())
}

fn num_from_bool(b:bool) -> u8 {
    if b {1} else {0}
}

impl DAQSample {
    pub fn encode_as_u8(&self) -> [u8; 16] {
        let mut encoded = [0xFF];
        let a = u16_to_bytes(self.trigger_id);
        let b = u64_to_bytes(self.trigger_clk);
        let c = u32_to_bytes(self.trigger_data);
        
        let mut z : u8 = 0;
        z = (num_from_bool(self.veto_in)<<7) ^ z;
        z = (num_from_bool(self.internal_trigger)<<6) ^ z;
        z = (1<<5) ^ z;
        z = (1<<4) ^ z;
        z = (1<<3) ^ z;
        z = (1<<2) ^ z;
        z = (1<<1) ^ z;
        
        let mut encoded: [u8; 3] = concat_arrays(encoded, a);
        let mut encoded: [u8; 11] = concat_arrays(encoded, b);
        let mut encoded: [u8; 15] = concat_arrays(encoded, c);
        let mut encoded: [u8; 16] = concat_arrays(encoded, [z]);
        encoded
    }
}

// u16, u64, u32, u1, u1
// -> 2(u8), 8(u8), 4(u8), 0.25(u8)
// -> 14.25(u8)
// 4 DAQSample -> 57(u8)

// determine length with postcard ??

//impl core::fmt::Debug for DAQSample {
//    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//        write!(f, "DAQSample {}, {}, {}, {}, {}", self.trigger_id, self.trigger_clk, self.trigger_data, self.veto_in, self.internal_trigger)
//    }
//}