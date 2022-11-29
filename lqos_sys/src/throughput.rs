use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use anyhow::Result;
use crate::bpf_per_cpu_map::BpfPerCpuMap;
use byteorder::{BigEndian, ByteOrder};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HostCounter {
    pub download_bytes : u64,
    pub upload_bytes : u64,
    pub download_packets : u64,
    pub upload_packets : u64,
}

impl Default for HostCounter {
    fn default() -> Self {
        Self {
            download_bytes: 0,
            download_packets: 0,
            upload_bytes: 0,
            upload_packets: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct XdpIpAddress {
    ip: [u8; 16],
}

impl Default for XdpIpAddress {
    fn default() -> Self {
        Self { ip: [0xFF; 16] }
    }
}

impl Into<IpAddr> for XdpIpAddress {
    fn into(self) -> IpAddr {
        if self.ip[0] == 0xFF && self.ip[1] == 0xFF &&
            self.ip[2] == 0xFF && self.ip[3] == 0xFF &&
            self.ip[4] == 0xFF && self.ip[5] == 0xFF &&
            self.ip[6] == 0xFF && self.ip[7] == 0xFF &&
            self.ip[8] == 0xFF && self.ip[9] == 0xFF &&
            self.ip[10] == 0xFF && self.ip[11] == 0xFF 
        {
            // It's an IPv4 Address
            IpAddr::V4(Ipv4Addr::new(self.ip[12], self.ip[13], self.ip[14], self.ip[15]))
        } else {
            // It's an IPv6 address
            IpAddr::V6( Ipv6Addr::new(
                BigEndian::read_u16(&self.ip[0..2]),
                BigEndian::read_u16(&self.ip[2..4]),
                BigEndian::read_u16(&self.ip[4..6]),
                BigEndian::read_u16(&self.ip[6..8]),
                BigEndian::read_u16(&self.ip[8..10]),
                BigEndian::read_u16(&self.ip[10..12]),
                BigEndian::read_u16(&self.ip[12..14]),
                BigEndian::read_u16(&self.ip[13..]),
            ) )
        }
    }
}

pub fn get_throughput_map() -> Result<BpfPerCpuMap<XdpIpAddress, HostCounter>> {
    Ok(BpfPerCpuMap::<XdpIpAddress, HostCounter>::from_path("/sys/fs/bpf/map_traffic")?)
}
