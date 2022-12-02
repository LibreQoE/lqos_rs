use anyhow::{Error, Result};
use clap::{Parser, Subcommand};
use lqos_bus::{decode_response, encode_request, BusRequest, BusSession, BUS_BIND_ADDRESS, BusResponse, IpMapping};
use std::{net::IpAddr, process::exit};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Parser)]
#[command()]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add an IP Address (v4 or v6) to the XDP/TC mapping system.
    Add {
        /// IP Address (v4 or v6) to add
        #[arg(long)]
        ip: String,

        /// TC Class ID (handle) to connect
        #[arg(long)]
        classid: String,

        /// CPU id to connect
        #[arg(long)]
        cpu: String,
    },
    /// Remove an IP address (v4 or v6) from the XDP/TC mapping system.
    Del {
        /// IP Address (v4 or v6) to remove
        ip: String,
    },
    /// Clear all mapped IPs.
    Clear,
    /// List all mapped IPs.
    List,
}

async fn talk_to_server(command: BusRequest) -> Result<()> {
    let mut stream = TcpStream::connect(BUS_BIND_ADDRESS).await?;
    let test = BusSession {
        auth_cookie: 1234,
        requests: vec![command],
    };
    let msg = encode_request(&test)?;
    stream.write(&msg).await?;
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf).await.unwrap();
    let reply = decode_response(&buf)?;
    match &reply.responses[0] {
        BusResponse::Ack => {
            println!("Success");
            Ok(())
        }
        BusResponse::Fail(err) => Err(Error::msg(err.clone())),
        BusResponse::MappedIps(ips) => {
            print_ips(&ips);
            Ok(())
        }
        _ => Err(Error::msg("Command execution failed")),
    }
}

fn print_ips(ips: &[IpMapping]) {
    println!("\nMapped IP Addresses:");
    println!("--------------------------------------------------------------------");
    for ip in ips.iter() {
        let ip_formatted = if ip.ip_address.contains(":") {
            format!("{}/{}", ip.ip_address, ip.prefix_length)
        } else {
            format!("{}/{}", ip.ip_address, ip.prefix_length-96)
        };
        let (major, minor) = (
            (ip.tc_handle & 0xFFFF0000) >> 16,
            ip.tc_handle & 0x0000FFFF,
        );
        println!("{:<45} CPU: {:<4} TC: {}:{}", ip_formatted, ip.cpu, major, minor);
    }
    println!("");
}

fn parse_add_ip(ip: &str, classid: &str, cpu: &str) -> Result<BusRequest> {
    if ip.parse::<IpAddr>().is_err() {
        return Err(Error::msg("Unable to parse IP address"));
    }
    if !classid.contains(":") {
        return Err(Error::msg(
            "Class id must be in the format (major):(minor), e.g. 1:12",
        ));
    }
    let split_class: Vec<&str> = classid.split(":").collect();
    Ok(BusRequest::MapIpToFlow {
        ip_address: ip.to_string(),
        tc_major: split_class[0].parse()?,
        tc_minor: split_class[1].parse()?,
        cpu: cpu.parse()?,
    })
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let cli = Args::parse();

    match cli.command {
        Some(Commands::Add { ip, classid, cpu }) => {
            talk_to_server(parse_add_ip(&ip, &classid, &cpu)?).await?;
        }
        Some(Commands::Del { ip }) => talk_to_server(BusRequest::DelIpFlow { ip_address: ip.to_string() }).await?,
        Some(Commands::Clear) => talk_to_server(BusRequest::ClearIpFlow).await?,
        Some(Commands::List) => talk_to_server(BusRequest::ListIpFlow).await?,
        None => {
            println!("Run with --help to see instructions");
            exit(0);
        }
    }

    Ok(())
}
