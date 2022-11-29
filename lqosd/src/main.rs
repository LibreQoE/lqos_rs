mod throughput_tracker;
use std::time::Duration;
use lqos_config::LibreQoSConfig;
use lqos_sys::LibreQoSKernels;

fn main() -> anyhow::Result<()> {
    println!("LibreQoS Daemon Starting");
    let config = LibreQoSConfig::load_from_default()?;
    let _kernels = LibreQoSKernels::new(&config.internet_interface, &config.isp_interface)?;
    let mut throughput = throughput_tracker::ThroughputTracker::new();

    loop {
        std::thread::sleep(Duration::from_secs(1));
        let _ = throughput.tick(); // Ignoring errors
        let bps = throughput.bits_per_second();
        let packets = throughput.packets_per_second();
        if throughput.cycle > 0 {
            println!("🠗 {} bits/second ({} packets), {} 🠕 bits/second ({} packets)", bps.0, packets.0, bps.1, packets.1);
        }
    }
}
