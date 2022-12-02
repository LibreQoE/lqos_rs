use crate::{bpf_per_cpu_map::BpfPerCpuMap, XdpIpAddress};
use anyhow::Result;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HostCounter {
    pub download_bytes: u64,
    pub upload_bytes: u64,
    pub download_packets: u64,
    pub upload_packets: u64,
    pub tc_handle: u32,
}

impl Default for HostCounter {
    fn default() -> Self {
        Self {
            download_bytes: 0,
            download_packets: 0,
            upload_bytes: 0,
            upload_packets: 0,
            tc_handle: 0,
        }
    }
}

/// Queries the underlying `map_traffic` eBPF pinned map, and returns every entry.
pub fn get_throughput_map() -> Result<Vec<(XdpIpAddress, Vec<HostCounter>)>> {
    Ok(BpfPerCpuMap::<XdpIpAddress, HostCounter>::from_path(
        "/sys/fs/bpf/map_traffic",
    )?.dump_vec())
}
