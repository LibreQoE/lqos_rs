mod cache_manager;
mod cache;
pub use cache::SHAPED_DEVICES;
pub use cache_manager::update_tracking;
use std::net::IpAddr;
use lqos_bus::IpStats;
use rocket::serde::json::Json;
use crate::tracker::cache::ThroughputPerSecond;
use self::cache::{CURRENT_THROUGHPUT, THROUGHPUT_BUFFER, CPU_USAGE, MEMORY_USAGE, TOP_10_DOWNLOADERS, WORST_10_RTT, RTT_HISTOGRAM, HOST_COUNTS};

#[get("/api/current_throughput")]
pub fn current_throughput() -> Json<ThroughputPerSecond> {
    let result = CURRENT_THROUGHPUT.read().clone();
    Json(result)
}

#[get("/api/throughput_ring")]
pub fn throughput_ring() -> Json<Vec<ThroughputPerSecond>> {
    let result = THROUGHPUT_BUFFER.read().get_result();
    Json(result)
}

#[get("/api/cpu")]
pub fn cpu_usage() -> Json<Vec<f32>> {
    let cpu_usage = CPU_USAGE.read().clone();

    Json(cpu_usage)
}

#[get("/api/ram")]
pub fn ram_usage() -> Json<Vec<u64>> {
    let ram_usage = MEMORY_USAGE.read().clone();
    Json(ram_usage)
}

#[get("/api/top_10_downloaders")]
pub fn top_10_downloaders() -> Json<Vec<IpStats>> {
    let mut tt = TOP_10_DOWNLOADERS.read().clone();
    let cfg = SHAPED_DEVICES.read();
    tt.iter_mut().for_each(|d| {
        if let Ok(ip) = d.ip_address.parse::<IpAddr>() {
            let lookup = match ip {
                IpAddr::V4(ip) => ip.to_ipv6_mapped(),
                IpAddr::V6(ip) => ip,
            };
            if let Some((_, id)) = cfg.trie.longest_match(lookup) {
                d.ip_address = format!("{} ({})", cfg.devices[*id].circuit_name, d.ip_address);
            }
        }
    });
    Json(tt)
}

#[get("/api/worst_10_rtt")]
pub fn worst_10_rtt() -> Json<Vec<IpStats>> {
    let mut tt = WORST_10_RTT.read().clone();
    let cfg = SHAPED_DEVICES.read();
    tt.iter_mut().for_each(|d| {
        if let Ok(ip) = d.ip_address.parse::<IpAddr>() {
            let lookup = match ip {
                IpAddr::V4(ip) => ip.to_ipv6_mapped(),
                IpAddr::V6(ip) => ip,
            };
            if let Some((_, id)) = cfg.trie.longest_match(lookup) {
                d.ip_address = format!("{} ({})", cfg.devices[*id].circuit_name, d.ip_address);
            }
        }
    });
    Json(tt)
}


#[get("/api/rtt_histogram")]
pub fn rtt_histogram() -> Json<Vec<u32>> {
    Json(RTT_HISTOGRAM.read().clone())
}

#[get("/api/host_counts")]
pub fn host_counts() -> Json<(u32, u32)> {
    let shaped_reader = SHAPED_DEVICES.read();
    let n_devices = shaped_reader.devices.len();
    let host_counts = HOST_COUNTS.read();
    let unknown = host_counts.0 - host_counts.1;
    Json((n_devices as u32, unknown))
}
