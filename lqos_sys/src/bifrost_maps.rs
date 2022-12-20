use anyhow::Result;
use lqos_config::{BridgeInterface, BridgeVlan};

use crate::{bpf_map::BpfMap, lqos_kernel::interface_name_to_index};

#[repr(C)]
#[derive(Default, Clone, Debug)]
struct BifrostInterface {
    redirect_to: u32,
    direction: u32,
}

#[repr(C)]
#[derive(Default, Clone, Debug)]
struct BifrostVlan {
    match_tag: u16,
    new_tag: u16,
    direction: u32,
    interface: u32,
}

const INTERFACE_PATH: &str = "/sys/fs/bpf/bifrost_interface_map";
const VLAN_PATH: &str = "/sys/fs/bpf/bifrost_vlan_map";

pub(crate) fn clear_bifrost() -> Result<()> {
    println!("Clearing bifrost maps");
    let mut interface_map = BpfMap::<u32, BifrostInterface>::from_path(INTERFACE_PATH)?;
    let mut vlan_map = BpfMap::<u32, BifrostVlan>::from_path(VLAN_PATH)?;
    println!("Clearing VLANs");
    vlan_map.clear_no_repeat()?;
    println!("Clearing Interfaces");
    interface_map.clear_no_repeat()?;
    println!("Done");
    Ok(())
}

pub(crate) fn map_interfaces(mappings: &[BridgeInterface]) -> Result<()> {
    println!("Interface maps");
    let mut interface_map = BpfMap::<u32, BifrostInterface>::from_path(INTERFACE_PATH)?;
    for mapping in mappings.iter() {
        // Key is the parent interface
        let mut from = interface_name_to_index(&mapping.name)?;
        let redirect_to = interface_name_to_index(&mapping.redirect_to)?;
        let mut mapping = BifrostInterface {
            redirect_to,
            direction: match mapping.interface_type {
                lqos_config::InterfaceFacing::Internet => 1,
                lqos_config::InterfaceFacing::Isp => 2,
                lqos_config::InterfaceFacing::Trunk => 3,
            }
        };
        interface_map.insert(&mut from, &mut mapping)?;
        println!("Mapped bifrost interface {}->{}", from, redirect_to);
    }
    Ok(())
}

pub(crate) fn map_vlans(mappings: &[BridgeVlan]) -> Result<()> {
    println!("VLAN maps");
    let mut vlan_map = BpfMap::<u32, BifrostVlan>::from_path(VLAN_PATH)?;
    for mapping in mappings.iter() {
        let mut from = mapping.internet_tag;
        let mut mapping = BifrostVlan {
            match_tag: mapping.internet_tag as u16,
            new_tag: mapping.isp_tag as u16,
            direction: 1,
            interface: interface_name_to_index(&mapping.redirect_to)?,
        };
        vlan_map.insert(&mut from, &mut mapping)?;
        println!("Mapped bifrost VLAN {}", from);
    }
    Ok(())
}