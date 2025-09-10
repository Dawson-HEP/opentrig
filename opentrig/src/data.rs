use core::ops::Deref;
use defmt::Format;
use serde::{Serialize, Deserialize};
use postcard::{from_bytes, to_vec};
use heapless::Vec;

use defmt;

#[derive(Serialize, Deserialize, Debug, Format, Eq, PartialEq)]
pub struct DAQSample {
    pub trigger_id: u16,
    pub trigger_clk: u64,
    pub trigger_data: u32,
    pub veto_in: bool,
    pub internal_trigger: bool,
}

//impl core::fmt::Debug for DAQSample {
//    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//        write!(f, "DAQSample {}, {}, {}, {}, {}", self.trigger_id, self.trigger_clk, self.trigger_data, self.veto_in, self.internal_trigger)
//    }
//}