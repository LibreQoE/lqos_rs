mod throughput_tracker;
use lqos_bus::{decode_request, BUS_BIND_ADDRESS, cookie_value, BusRequest, BusReply, encode_response};
use lqos_config::LibreQoSConfig;
use lqos_sys::LibreQoSKernels;
use anyhow::Result;
use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}};

#[tokio::main]
async fn main() -> Result<()> {
    println!("LibreQoS Daemon Starting");
    let config = LibreQoSConfig::load_from_default()?;
    let _kernels = LibreQoSKernels::new(&config.internet_interface, &config.isp_interface)?;
    throughput_tracker::spawn_throughput_monitor().await;

    // Main bus listen loop
    let listener = TcpListener::bind(BUS_BIND_ADDRESS).await?;
    println!("Listening on: {}", BUS_BIND_ADDRESS);
    loop {
        let (mut socket, _) = listener.accept().await?;
            tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            let _ = socket
                .read(&mut buf)
                .await
                .expect("failed to read data from socket");
            
            if let Ok(request) = decode_request(&buf) {
                if request.auth_cookie == cookie_value() {
                    let mut response = BusReply {
                        auth_cookie: request.auth_cookie,
                        responses: Vec::new(),
                    };
                    for req in request.requests.iter() {
                        response.responses.push(
                        match req {
                            BusRequest::Ping => lqos_bus::BusResponse::Ack,
                            BusRequest::GetCurrentThroughput => throughput_tracker::current_throughput(),
                            BusRequest::GetTopNDownloaders(n) => throughput_tracker::top_n(*n),
                        });
                    }
                    let _ = reply(&encode_response(&response).unwrap(), &mut socket).await;
                }
            }
        });
    }

    /*
    let mut throughput = throughput_tracker::ThroughputTracker::new();

    add_ip_to_tc("100.64.1.2/32", (1, 12), 2)?;

    loop {
        std::thread::sleep(Duration::from_secs(1));
        let _ = throughput.tick(); // Ignoring errors
        let bps = throughput.bits_per_second();
        let packets = throughput.packets_per_second();
        if throughput.cycle > 1 {
            println!("🠗 {} bits/second ({} packets), {} 🠕 bits/second ({} packets)", bps.0, packets.0, bps.1, packets.1);
        }
        throughput.dump();
    }*/
}

async fn reply(response: &[u8], socket: &mut TcpStream) -> Result<()> {
    socket.write_all(&response).await?;
    Ok(())
}