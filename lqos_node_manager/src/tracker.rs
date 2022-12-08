use std::time::Duration;
use anyhow::Result;
use lqos_bus::{BUS_BIND_ADDRESS, BusRequest, BusSession, encode_request, decode_response, BusResponse, IpStats};
use rocket::tokio::{net::TcpStream, io::{AsyncWriteExt, AsyncReadExt}};
use rocket::serde::{json::Json, Deserialize, Serialize};
use parking_lot::RwLock;
use lazy_static::*;

pub async fn update_tracking() {
    use sysinfo::System;
    use sysinfo::CpuExt;
    use sysinfo::SystemExt;
    let mut sys = System::new_all();

    loop {
        println!("Updating tracking data");
        sys.refresh_cpu();
        sys.refresh_memory();
        let cpu_usage = sys
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .collect::<Vec<f32>>();
        *CPU_USAGE.write() = cpu_usage;
        {
            let mut mem_use = MEMORY_USAGE.write();
            mem_use[0] = sys.used_memory();
            mem_use[1] = sys.total_memory();
        }
        let _ = get_data_from_server().await; // Ignoring errors to keep running
        rocket::tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ThroughputPerSecond {
    pub bits_per_second: (u64, u64),
    pub packets_per_second: (u64, u64),
}

impl Default for ThroughputPerSecond {
    fn default() -> Self {
        Self {
            bits_per_second: (0,0),
            packets_per_second: (0,0),
        }
    }
}

const RINGBUFFER_SAMPLES: usize = 300;

pub struct ThroughputRingbuffer {
    readings: Vec<ThroughputPerSecond>,
    next: usize,
}

impl ThroughputRingbuffer {
    fn new() -> Self {
        Self {
            readings: vec![ThroughputPerSecond::default(); RINGBUFFER_SAMPLES],
            next: 0,
        }
    }

    fn store(&mut self, reading: ThroughputPerSecond) {
        self.readings[self.next] = reading;
        self.next += 1;
        self.next %= RINGBUFFER_SAMPLES;
    }

    fn get_result(&self) -> Vec<ThroughputPerSecond> {
        let mut result = Vec::new();

        for i in self.next .. RINGBUFFER_SAMPLES {
            result.push(self.readings[i]);
        }
        for i in 0..self.next {
            result.push(self.readings[i]);
        }

        result
    }
}

lazy_static! {
    static ref CURRENT_THROUGHPUT : RwLock<ThroughputPerSecond> = RwLock::new(ThroughputPerSecond::default());
}

lazy_static! {
    static ref THROUGHPUT_BUFFER : RwLock<ThroughputRingbuffer> = RwLock::new(ThroughputRingbuffer::new());
}

lazy_static! {
    static ref CPU_USAGE : RwLock<Vec<f32>> = RwLock::new(Vec::new());
}

lazy_static! {
    static ref MEMORY_USAGE : RwLock<Vec<u64>> = RwLock::new(vec![0, 0]);
}

lazy_static! {
    static ref TOP_10_DOWNLOADERS : RwLock<Vec<IpStats>> = RwLock::new(Vec::new());
}

async fn get_data_from_server() -> Result<()> {
    // Send request to lqosd
    let mut stream = TcpStream::connect(BUS_BIND_ADDRESS).await?;
    let test = BusSession {
        auth_cookie: 1234,
        requests: vec![
            BusRequest::GetCurrentThroughput,
            BusRequest::GetTopNDownloaders(10),
        ],
    };
    let msg = encode_request(&test)?;
    stream.write(&msg).await?;

    // Receive reply
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf).await.unwrap();
    let reply = decode_response(&buf)?;

    // Process the reply
    for r in reply.responses.iter() {
        match r {
            BusResponse::CurrentThroughput {
                bits_per_second,
                packets_per_second,
            } => {
                {
                    let mut lock = CURRENT_THROUGHPUT.write();
                    lock.bits_per_second = *bits_per_second;
                    lock.packets_per_second = *packets_per_second;
                } // Lock scope
                {
                    let mut lock = THROUGHPUT_BUFFER.write();
                    lock.store(ThroughputPerSecond {
                        packets_per_second: *packets_per_second,
                        bits_per_second: *bits_per_second,
                    });
                }
            }
            BusResponse::TopDownloaders(stats) => {
                *TOP_10_DOWNLOADERS.write() = stats.clone();
            }
            // Default
            _ => {}
        }
    }

    Ok(())
}

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
    Json(TOP_10_DOWNLOADERS.read().clone())
}