use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IpStats {
    pub ip_address: String,
    pub bits_per_second: (u64, u64),
    pub packets_per_second: (u64, u64),
    pub median_tcp_rtt: f32,    
}