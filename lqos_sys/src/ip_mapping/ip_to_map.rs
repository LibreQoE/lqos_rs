use std::net::{IpAddr, Ipv6Addr, Ipv4Addr};
use anyhow::Result;

pub(crate) struct IpToMap {
    pub(crate) subnet: IpAddr,
    pub(crate) prefix: u32,
    pub(crate) tc_handle: (u16, u16),
    pub(crate) cpu: u32,
}

impl IpToMap {
    pub(crate) fn new(address: &str, tc_handle: (u16, u16), cpu: u32) -> Result<Self> {
        let address_part; // Filled in later
        let mut subnet_part = 128;
        if address.contains("/") {
            let parts: Vec<&str> = address.split('/').collect();
            address_part = parts[0].to_string();
            subnet_part = parts[1].replace("/", "").parse()?;
        } else {
            address_part = address.to_string();
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

    pub(crate) fn handle(&self) -> u32 {
        (self.tc_handle.0 as u32) << 16 | self.tc_handle.1 as u32
    }
}