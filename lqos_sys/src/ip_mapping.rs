use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use anyhow::Result;

use crate::{bpf_map::BpfMap, XdpIpAddress};

pub struct IpToMap {
    subnet: IpAddr,
    prefix: u32,
    tc_handle: (u16, u16),
    cpu: u32,
}

impl IpToMap {
    pub fn new(address: &str, tc_handle: (u16, u16), cpu: u32) -> Result<Self> {

        let mut address_part = String::new();
        let mut subnet_part = 128;
        if address.contains("/") {
            let parts : Vec<&str> = address.split('/').collect();
            address_part = parts[0].to_string();
            subnet_part = parts[1].replace("/", "").parse()?;
        }

        let subnet = if address_part.contains(":") {
            // It's an IPv6
            let ipv6 = address_part.parse::<Ipv6Addr>()?;
            IpAddr::V6(ipv6)
        } else {
            // It's an IPv4
            if subnet_part != 128 {
                subnet_part += 96;
            }
            let ipv4 = address_part.parse::<Ipv4Addr>()?;
            IpAddr::V4(ipv4)
        };

        Ok(Self {
            subnet,
            prefix: subnet_part,
            tc_handle,
            cpu,
        })
    }

    fn handle(&self) -> u32 {
        (self.tc_handle.0 as u32) << 16 | self.tc_handle.1 as u32
    }
}

#[repr(C)]
#[derive(Clone)]
struct IpHashData {
	cpu : u32,
	tc_handle : u32,
}

impl Default for IpHashData {
    fn default() -> Self {
        Self { cpu: 0, tc_handle: 0 }
    }
}

#[repr(C)]
#[derive(Clone)]
struct IpHashKey {
	prefixlen: u32,
	address: [u8; 16],
}

impl Default for IpHashKey {
    fn default() -> Self {
        Self { prefixlen: 0, address: [0xFF; 16] }
    }
}

pub fn add_ip_to_tc(address: &str, tc_handle: (u16, u16), cpu: u32) -> Result<()> {
    let ip_to_add = IpToMap::new(address, tc_handle, cpu)?;
    let mut bpf_map = BpfMap::<IpHashKey, IpHashData>::from_path("/sys/fs/bpf/map_ip_to_cpu_and_tc")?;
    let address = XdpIpAddress::from_ip(ip_to_add.subnet);
    let mut key = IpHashKey {
        prefixlen: ip_to_add.prefix,
        address: address.ip,
    };
    let mut value = IpHashData { cpu: ip_to_add.cpu, tc_handle: ip_to_add.handle() };
    bpf_map.insert(&mut key, &mut value)?;
    Ok(())
}