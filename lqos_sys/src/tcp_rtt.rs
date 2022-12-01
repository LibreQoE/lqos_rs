use anyhow::Result;

use crate::{bpf_map::BpfMap, XdpIpAddress};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RttTrackingEntry {
    pub tc_handle: u32,
    pub rtt: [u32; 60],
    pub next_entry: u32,
    pub recycle_time: u64,
    pub has_fresh_data: u32
}

impl Default for RttTrackingEntry {
    fn default() -> Self {
        Self {
            tc_handle: 0,
            rtt: [0; 60],
            next_entry: 0,
            recycle_time: 0,
            has_fresh_data: 0,
        }
    }
}

pub fn get_tcp_round_trip_times() -> Result<()> {
    let rtt_tracker = BpfMap::<XdpIpAddress, RttTrackingEntry>::from_path("/sys/fs/bpf/rtt_tracker")?;
    //println!("Tick");
    for rtt in rtt_tracker.iter() {
        println!("{:?}", rtt);
    }
    Ok(())
}