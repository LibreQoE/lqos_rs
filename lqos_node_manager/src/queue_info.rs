use lqos_bus::{BusResponse, BUS_BIND_ADDRESS, BusSession, BusRequest, encode_request, decode_response};
use rocket::response::content::RawJson;
use rocket::tokio::io::{AsyncWriteExt, AsyncReadExt};
use rocket::tokio::net::TcpStream;
use crate::cache_control::NoCache;

#[get("/api/raw_queue_by_circuit/<circuit_id>")]
pub async fn raw_queue_by_circuit(circuit_id: String) -> NoCache<RawJson<String>> {
    let mut stream = TcpStream::connect(BUS_BIND_ADDRESS).await.unwrap();
    let test = BusSession {
        auth_cookie: 1234,
        requests: vec![
            BusRequest::GetRawQueueData(circuit_id),
        ],
    };
    let msg = encode_request(&test).unwrap();
    stream.write(&msg).await.unwrap();

    // Receive reply
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf).await.unwrap();
    let reply = decode_response(&buf).unwrap();

    let result = match &reply.responses[0] {
        BusResponse::RawQueueData(msg) => msg.clone(),
        _ => "Unable to reload LibreQoS".to_string(),
    };
    NoCache::new(RawJson(result))
}