use anyhow::Result;
use lqos_bus::BusResponse;
use lqos_sys::{get_throughput_map, XdpIpAddress};
use tokio::{task, time};
use std::{collections::HashMap, time::Duration};
use lazy_static::*;
use parking_lot::RwLock;

lazy_static! {
    static ref THROUGHPUT_TRACKER : RwLock<ThroughputTracker> = RwLock::new(ThroughputTracker::new());
}

pub async fn spawn_throughput_monitor() {
    let _ = task::spawn(async {
        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            let _ = task::spawn_blocking(move || {
                let mut thoughput = THROUGHPUT_TRACKER.write();
                let _ = thoughput.tick();
            }).await;
            interval.tick().await;
        }
    });
}

pub fn current_throughput() -> BusResponse {
    let (bits_per_second, packets_per_second) = {
        let tp = THROUGHPUT_TRACKER.read();
        (tp.bits_per_second(), tp.packets_per_second())
    };
    BusResponse::CurrentThroughput { bits_per_second, packets_per_second }
}

pub fn top_n(n: u32) -> BusResponse {
    let mut full_list : Vec<(XdpIpAddress, (u64, u64), (u64, u64), f32)> = {
        let tp = THROUGHPUT_TRACKER.read();
        tp.raw_data.iter().map(|(ip, te)| {
            (
                *ip,
                te.bytes_per_second,
                te.packets_per_second,
                te.median_latency(),
            )
        }).collect()
    };
    full_list.sort_by(|a,b| {
        b.1.0.cmp(&a.1.0)
    });
    let result = full_list
        .iter()
        .take(n as usize)
        .map(|(ip, (bytes_dn, bytes_up), (packets_dn, packets_up ), median_rtt)| {
        (
            ip.as_ip().to_string(),
            (bytes_dn * 8, bytes_up * 8),
            (*packets_dn, *packets_up),
            *median_rtt
        )
    }).collect();
    BusResponse::TopDownloaders(result)
}

pub struct ThroughputTracker {
    pub cycle: u64,
    raw_data: HashMap<XdpIpAddress, ThroughputEntry>,
    bytes_per_second: (u64, u64),
    packets_per_second: (u64, u64),
}

impl ThroughputTracker {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            raw_data: HashMap::new(),
            bytes_per_second: (0, 0),
            packets_per_second: (0, 0),
        }
    }

    pub fn tick(&mut self) -> Result<()> {
        let throughput = get_throughput_map()?;
        let value_dump = throughput.dump_vec();
        
        // Copy previous byte/packet numbers and reset RTT data
        self.raw_data
            .iter_mut()
            .for_each(|(_k, v)| {
                if v.first_cycle < self.cycle {
                    v.bytes_per_second.0 = v.bytes.0 - v.prev_bytes.0;
                    v.bytes_per_second.1 = v.bytes.1 - v.prev_bytes.1;
                    v.packets_per_second.0 = v.packets.0 - v.prev_packets.0;
                    v.packets_per_second.1 = v.packets.1 - v.prev_packets.1;
                    v.prev_bytes = v.bytes;
                    v.prev_packets = v.packets;
                }
                //v.recent_rtt_data = [0; 60];
            });
        
        value_dump
            .iter()
            .for_each(|(xdp_ip, counts)| {
                if let Some(entry) = self.raw_data.get_mut(xdp_ip) {
                    entry.bytes = (0,0);
                    entry.packets = (0,0);
                    for c in counts {
                        entry.bytes.0 += c.download_bytes;
                        entry.bytes.1 += c.upload_bytes;
                        entry.packets.0 += c.download_packets;
                        entry.packets.1 += c.upload_packets;
                        if c.tc_handle != 0 {
                            entry.tc_handle = c.tc_handle;
                        }
                    };
                } else {
                    let mut entry = ThroughputEntry {
                        first_cycle: self.cycle,
                        bytes: (0, 0),
                        packets: (0, 0),
                        prev_bytes: (0, 0),
                        prev_packets: (0, 0),
                        bytes_per_second: (0, 0),
                        packets_per_second: (0, 0),
                        tc_handle: 0,
                        recent_rtt_data: [0; 60],
                    };
                    for c in counts {
                        entry.bytes.0 += c.download_bytes;
                        entry.bytes.1 += c.upload_bytes;
                        entry.packets.0 += c.download_packets;
                        entry.packets.1 += c.upload_packets;
                        if c.tc_handle != 0 {
                            entry.tc_handle = c.tc_handle;
                        }
                    };
                    self.raw_data.insert(*xdp_ip, entry);
                }
            });

        // Apply RTT data
        if let Ok(rtt_dump) = lqos_sys::get_tcp_round_trip_times() {
            for (raw_ip, rtt) in rtt_dump {
                if rtt.has_fresh_data != 0 {
                    let ip = XdpIpAddress{ ip: raw_ip };
                    if let Some(tracker) = self.raw_data.get_mut(&ip) {
                        tracker.recent_rtt_data = rtt.rtt;
                    }
                }
            }
        }

        // Update totals
        self.bytes_per_second = (0,0);
        self.packets_per_second = (0,0);
        self.raw_data
            .iter()
            .map(|(_k, v)| {
                (
                    v.bytes.0 - v.prev_bytes.0,
                    v.bytes.1 - v.prev_bytes.1,
                    v.packets.0 - v.prev_packets.0,
                    v.packets.1 - v.prev_packets.1,
                )
            })
            .for_each(|(bytes_down, bytes_up, packets_down, packets_up)| {
                self.bytes_per_second.0 += bytes_down;
                self.bytes_per_second.1 += bytes_up;
                self.packets_per_second.0 += packets_down;
                self.packets_per_second.1 += packets_up;
            });

        // Onto the next cycle
        self.cycle += 1;
        Ok(())
    }

    pub fn bits_per_second(&self) -> (u64, u64) {
        (self.bytes_per_second.0 * 8, self.bytes_per_second.1 * 8)
    }

    pub fn packets_per_second(&self) -> (u64, u64) {
        self.packets_per_second
    }

    #[allow(dead_code)]
    pub fn dump(&self) {
        for (k,v) in self.raw_data.iter() {
            let ip = k.as_ip();
            println!("{:<34}{:?}", ip, v.tc_handle);
        }
    }
}

#[derive(Debug)]
struct ThroughputEntry {
    first_cycle: u64,
    bytes: (u64, u64),
    packets: (u64, u64),
    prev_bytes: (u64, u64),
    prev_packets: (u64, u64),
    bytes_per_second: (u64, u64),
    packets_per_second: (u64, u64),
    tc_handle: u32,
    recent_rtt_data: [u32; 60],
}

impl ThroughputEntry {
    fn median_latency(&self) -> f32 {
        let mut shifted : Vec<f32> = self.recent_rtt_data.iter().filter(|n| **n != 0).map(|n| *n as f32 / 100.0).collect();
        if shifted.is_empty() {
            return 0.0;
        }
        shifted.sort_by(|a,b| a.partial_cmp(&b).unwrap());
        shifted[shifted.len() / 2]
    }
}