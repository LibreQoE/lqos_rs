use serde::{Deserialize, Serialize};
use anyhow::Result;

pub const BUS_BIND_ADDRESS : &str = "127.0.0.1:9999";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BusSession {
    pub auth_cookie: u32,
    pub requests: Vec<BusRequest>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BusRequest {
    Ping, // A generic "is it alive" test
    GetCurrentThroughput,
    GetTopNDownloaders(u32),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BusReply {
    pub auth_cookie: u32,
    pub responses: Vec<BusResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BusResponse {
    Ack, // Yes, we're alive
    CurrentThroughput{ bits_per_second: (u64, u64), packets_per_second: (u64, u64) },
    TopDownloaders(Vec<(String, (u64, u64), (u64, u64))>),
}

pub fn encode_request(request: &BusSession) -> Result<Vec<u8>> {
    Ok(bincode::serialize(request)?)
}

pub fn decode_request(bytes: &[u8]) -> Result<BusSession> {
    Ok(bincode::deserialize(&bytes)?)
}

pub fn encode_response(request: &BusReply) -> Result<Vec<u8>> {
    Ok(bincode::serialize(request)?)
}

pub fn decode_response(bytes: &[u8]) -> Result<BusReply> {
    Ok(bincode::deserialize(&bytes)?)
}

pub fn cookie_value() -> u32 {
    1234
}