use tokio::process::Command;

pub async fn stop_irq_balance() {
    let _ = Command::new("/bin/systemctl")
        .args(["stop", "irqbalance"])
        .output().await;
}

pub async fn netdev_budget(usecs: u32, packets: u32) {
    let _ = Command::new("/sbin/sysctl")
        .arg(format!("net.core.netdev_budget_usecs={usecs}"))
        .output().await;

    let _ = Command::new("/sbin/sysctl")
        .arg(format!("net.core.netdev_budget={packets}"))
        .output().await;
}

async fn disable_individual_offload(interface: &str, feature: &str) {
    let _ = Command::new("/sbin/ethtool")
        .args(["--offload", interface, feature, "off"])
        .output().await;
}

pub async fn ethtool_tweaks(interface: &str) {
    // Disabling individually to avoid complaints that a card doesn't support a feature anyway
    let features = ["gso", "tso", "lro", "sg", "gro"];
    for feature in features.iter() {
        disable_individual_offload(interface, feature).await;
    }
    
    let _ = Command::new("/sbin/ethtool")
        .args(["-C", interface, "rx-usecs", "0"])
        .output().await;

    let _ = Command::new("/sbin/ethtool")
        .args(["-C", interface, "tx-usecs", "0"])
        .output().await;

    let _ = Command::new("/sbin/ethtool")
        .args(["-K", interface, "rxvlan", "off"])
        .output().await;

    let _ = Command::new("/sbin/ethtool")
        .args(["-K", interface, "txvlan", "off"])
        .output().await;
}