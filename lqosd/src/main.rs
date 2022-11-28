use std::time::Duration;
use lqos_config::LibreQoSConfig;
use lqos_sys::{attach_xdp_to_interface, InterfaceDirection, unload_xdp_from_interface, BpfMapReader};

fn main() -> anyhow::Result<()> {
    println!("LibreQoS Daemon Starting");
    let config = LibreQoSConfig::load_from_default()?;
    unload_xdp_from_interface("eth0")?;
    attach_xdp_to_interface(&config.internet_interface, InterfaceDirection::Internet)?;
    attach_xdp_to_interface(&config.isp_interface, InterfaceDirection::IspNetwork)?;

    loop {
        std::thread::sleep(Duration::from_secs(1));
        let test = BpfMapReader::open("/sys/fs/bpf/map_traffic")?;
        test.test();
        println!("-");
    }
}
