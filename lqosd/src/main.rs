use lqos_config::LibreQoSConfig;
use lqos_sys::{attach_xdp_to_interface, InterfaceDirection, unload_xdp_from_interface};

fn main() -> anyhow::Result<()> {
    println!("LibreQoS Daemon Starting");
    let config = LibreQoSConfig::load_from_default()?;
    unload_xdp_from_interface("eth0")?;
    attach_xdp_to_interface(&config.internet_interface, InterfaceDirection::Internet)?;
    attach_xdp_to_interface(&config.isp_interface, InterfaceDirection::IspNetwork)?;
    Ok(())
}
