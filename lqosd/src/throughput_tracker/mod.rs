mod tracking_data;
mod throughput_entry;
use lazy_static::*;
use lqos_bus::{BusResponse, IpStats, XdpPpingResult};
use lqos_sys::XdpIpAddress;
use parking_lot::RwLock;
use std::time::Duration;
use tokio::{task, time};
use crate::throughput_tracker::tracking_data::ThroughputTracker;

lazy_static! {
    static ref THROUGHPUT_TRACKER: RwLock<ThroughputTracker> =
        RwLock::new(ThroughputTracker::new());
}

pub async fn spawn_throughput_monitor() {
    let _ = task::spawn(async {
        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            let _ = task::spawn_blocking(move || {
                let mut thoughput = THROUGHPUT_TRACKER.write();
                let _ = thoughput.tick();
            })
            .await;
            interval.tick().await;
        }
    });
}

pub fn current_throughput() -> BusResponse {
    let (bits_per_second, packets_per_second) = {
        let tp = THROUGHPUT_TRACKER.read();
        (tp.bits_per_second(), tp.packets_per_second())
    };
    BusResponse::CurrentThroughput {
        bits_per_second,
        packets_per_second,
    }
}

pub fn top_n(n: u32) -> BusResponse {
    let mut full_list: Vec<(XdpIpAddress, (u64, u64), (u64, u64), f32)> = {
        let tp = THROUGHPUT_TRACKER.read();
        tp.raw_data
            .iter()
            .map(|(ip, te)| {
                (
                    *ip,
                    te.bytes_per_second,
                    te.packets_per_second,
                    te.median_latency(),
                )
            })
            .collect()
    };
    full_list.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
    let result = full_list
        .iter()
        .take(n as usize)
        .map(
            |(ip, (bytes_dn, bytes_up), (packets_dn, packets_up), median_rtt)| IpStats {
                ip_address: ip.as_ip().to_string(),
                bits_per_second: (bytes_dn * 8, bytes_up * 8),
                packets_per_second: (*packets_dn, *packets_up),
                median_tcp_rtt: *median_rtt,
            },
        )
        .collect();
    BusResponse::TopDownloaders(result)
}

pub fn xdp_pping_compat() -> BusResponse {
    let raw = THROUGHPUT_TRACKER.read();
    let result = raw.raw_data
        .iter()
        .filter_map(|(_ip, data)| {
            if data.tc_handle > 0 {
                let mut valid_samples : Vec<u32> = data.recent_rtt_data.iter().filter(|d| **d > 0).map(|d| *d).collect();
                let samples = valid_samples.len() as u32;
                if samples > 0 {
                    valid_samples.sort_by(|a,b| (*a).cmp(&b));
                    let median = valid_samples[valid_samples.len() / 2] as f32 / 100.0;
                    let max = *(valid_samples.iter().max().unwrap()) as f32 / 100.0;
                    let min = *(valid_samples.iter().min().unwrap()) as f32 / 100.0;
                    let sum = valid_samples.iter().sum::<u32>() as f32 / 100.0;
                    let avg = sum / samples as f32;

                    Some(XdpPpingResult {
                        tc: format!("{}:{}", (data.tc_handle & 0xFFFF0000) >> 16, data.tc_handle & 0x0000FFFF),
                        median,
                        avg,
                        max,
                        min,
                        samples
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    BusResponse::XdpPping(result)
}
