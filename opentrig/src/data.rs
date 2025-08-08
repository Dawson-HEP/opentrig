pub struct DAQSample {
    pub trigger_id: u16,
    pub trigger_clk: u64,
    pub trigger_data: u32,
    pub veto_in: bool,
    pub internal_trigger: bool,
}
