use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IpStats {
    pub ip_address: String,
    pub bits_per_second: (u64, u64),
    pub packets_per_second: (u64, u64),
    pub median_tcp_rtt: f32,    
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IpMapping {
    pub ip_address: String,
    pub prefix_length: u32,
    pub tc_handle: u32,
    pub cpu: u32,
}