use std::{time::Duration, net::IpAddr};
use lqos_config::LibreQoSConfig;
use lqos_sys::{attach_xdp_to_interface, InterfaceDirection, get_throughput_map};

fn main() -> anyhow::Result<()> {
    println!("LibreQoS Daemon Starting");
    let config = LibreQoSConfig::load_from_default()?;
    attach_xdp_to_interface(&config.internet_interface, InterfaceDirection::Internet)?;
    attach_xdp_to_interface(&config.isp_interface, InterfaceDirection::IspNetwork)?;

    loop {
        std::thread::sleep(Duration::from_secs(1));
        let throughput = get_throughput_map().unwrap();
        for (ip, c) in throughput.iter() {
            let ip: IpAddr = ip.into();
            println!("{:<34}, 🠗 {} ({}), 🠕 {} ({})", ip, c.download_bytes, c.download_packets, c.upload_bytes, c.upload_packets);
        }
        println!("----------------------------------------------------------");
    }
}
