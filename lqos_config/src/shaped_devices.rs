use std::{path::Path, net::{Ipv4Addr, Ipv6Addr}};
use anyhow::{Result, Error};
use csv::StringRecord;
use crate::etc;

pub struct ConfigShapedDevices {
    pub devices: Vec<ShapedDevice>,
    pub trie: ip_network_table::IpNetworkTable<usize>,
}

impl ConfigShapedDevices {
    pub fn load() -> Result<Self> {
        let cfg = etc::EtcLqos::load()?;
        let base_path = Path::new(&cfg.lqos_directory);
        let final_path = base_path.join("ShapedDevices.csv");
        let mut reader = csv::Reader::from_path(final_path)?;
        
        // Example: StringRecord(["1", "968 Circle St., Gurnee, IL 60031", "1", "Device 1", "", "", "192.168.101.2", "", "25", "5", "10000", "10000", ""])
        let mut devices = Vec::new();
        for result in reader.records() {
            if let Ok(result) = result {
                if let Ok(device) = ShapedDevice::from_csv(&result) {
                    devices.push(device);
                }
            }
        }
        let trie = ConfigShapedDevices::make_trie(&devices);
        Ok(Self{ devices, trie })
    }

    fn make_trie(devices: &[ShapedDevice]) -> ip_network_table::IpNetworkTable<usize> {
        use ip_network::IpNetwork;
        let mut table = ip_network_table::IpNetworkTable::new();
        devices
            .iter()
            .enumerate()
            .map(|(i,d)| { (i, d.to_ipv6_list()) })
            .for_each(|(id, ips)| {
                ips.iter().for_each(|(ip, cidr)| {
                    if let Ok(net) = IpNetwork::new(*ip, (*cidr) as u8) {
                        table.insert(net, id);
                    }
                });
            });
        table
    }
}

#[derive(Clone, Debug)]
pub struct ShapedDevice {
    // Circuit ID,Circuit Name,Device ID,Device Name,Parent Node,MAC,IPv4,IPv6,Download Min Mbps,Upload Min Mbps,Download Max Mbps,Upload Max Mbps,Comment
    pub circuit_id: String,
    pub circuit_name: String,
    pub device_id: String,
    pub device_name: String,
    pub parent_node: String,
    pub mac: String,
    pub ipv4: Vec<(Ipv4Addr, u32)>,
    pub ipv6: Vec<(Ipv6Addr, u32)>,
    pub download_min_mbps: u32,
    pub upload_min_mbps: u32,
    pub download_max_mbps: u32,
    pub upload_max_mbps: u32,
    pub comment: String,
}

impl Default for ShapedDevice {
    fn default() -> Self {
        Self {
            circuit_id: String::new(),
            circuit_name: String::new(),
            device_id: String::new(),
            device_name: String::new(),
            parent_node: String::new(),
            mac: String::new(),
            ipv4: Vec::new(),
            ipv6: Vec::new(),
            download_min_mbps: 0,
            download_max_mbps: 0,
            upload_min_mbps: 0,
            upload_max_mbps: 0,
            comment: String::new(),
        }
    }
}

impl ShapedDevice {
    fn from_csv(record: &StringRecord) -> Result<Self> {
        Ok(Self {
            circuit_id: record[0].to_string(),
            circuit_name: record[1].to_string(),
            device_id: record[2].to_string(),
            device_name: record[3].to_string(),
            parent_node: record[4].to_string(),
            mac: record[5].to_string(),
            ipv4: ShapedDevice::parse_ipv4(&record[6]),
            ipv6: ShapedDevice::parse_ipv6(&record[7]),
            download_min_mbps: record[8].parse()?,
            upload_min_mbps: record[9].parse()?,
            download_max_mbps: record[10].parse()?,
            upload_max_mbps: record[11].parse()?,
            comment: record[12].to_string(),
        })
    }

    fn parse_cidr_v4(address: &str) -> Result<(Ipv4Addr, u32)> {
        if address.contains("/") {
            let split : Vec<&str> = address.split("/").collect();
            if split.len() != 2 {
                return Err(Error::msg("Unable to parse IPv4"));
            }
            return Ok((
                split[0].parse()?,
                split[1].parse()?
            ))
        } else {
            return Ok((
                address.parse()?,
                32
            ));
        }
    }

    fn parse_ipv4(str: &str) -> Vec<(Ipv4Addr, u32)> {
        let mut result = Vec::new();
        if str.contains(",") {
            for ip in str.split(",") {
                let ip = ip.trim();
                if let Ok((ipv4, subnet)) = ShapedDevice::parse_cidr_v4(ip) {
                    result.push((ipv4, subnet));
                }
            }
        } else {
            // No Commas
            if let Ok((ipv4, subnet)) = ShapedDevice::parse_cidr_v4(str) {
                result.push((ipv4, subnet));
            }
        }

        result
    }

    fn parse_cidr_v6(address: &str) -> Result<(Ipv6Addr, u32)> {
        if address.contains("/") {
            let split : Vec<&str> = address.split("/").collect();
            if split.len() != 2 {
                return Err(Error::msg("Unable to parse IPv6"));
            }
            return Ok((
                split[0].parse()?,
                split[1].parse()?
            ))
        } else {
            return Ok((
                address.parse()?,
                128
            ));
        }
    }

    fn parse_ipv6(str: &str) -> Vec<(Ipv6Addr, u32)> {
        let mut result = Vec::new();
        if str.contains(",") {
            for ip in str.split(",") {
                let ip = ip.trim();
                if let Ok((ipv6, subnet)) = ShapedDevice::parse_cidr_v6(ip) {
                    result.push((ipv6, subnet));
                }
            }
        } else {
            // No Commas
            if let Ok((ipv6, subnet)) = ShapedDevice::parse_cidr_v6(str) {
                result.push((ipv6, subnet));
            }
        }

        result
    }

    fn to_ipv6_list(&self) -> Vec<(Ipv6Addr, u32)> {
        let mut result = Vec::new();

        for (ipv4, cidr) in &self.ipv4 {
            result.push((
                ipv4.to_ipv6_mapped(),
                cidr + 96
            ));
        }
        result.extend_from_slice(&self.ipv6);

        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple_ipv4_parse() {
        let (ip, cidr) = ShapedDevice::parse_cidr_v4("1.2.3.4").unwrap();
        assert_eq!(cidr, 32);
        assert_eq!("1.2.3.4".parse::<Ipv4Addr>().unwrap(), ip);
    }

    #[test]
    fn test_cidr_ipv4_parse() {
        let (ip, cidr) = ShapedDevice::parse_cidr_v4("1.2.3.4/24").unwrap();
        assert_eq!(cidr, 24);
        assert_eq!("1.2.3.4".parse::<Ipv4Addr>().unwrap(), ip);
    }

    #[test]
    fn test_bad_ipv4_parse() {
        let r = ShapedDevice::parse_cidr_v4("bad wolf");
        assert!(r.is_err());
    }

    #[test]
    fn test_nearly_ok_ipv4_parse() {
        let r = ShapedDevice::parse_cidr_v4("192.168.1.256/32");
        assert!(r.is_err());
    }

    #[test]
    fn test_single_ipv4() {
        let r = ShapedDevice::parse_ipv4("1.2.3.4");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].0, "1.2.3.4".parse::<Ipv4Addr>().unwrap());
        assert_eq!(r[0].1, 32);
    }

    #[test]
    fn test_two_ipv4() {
        let r = ShapedDevice::parse_ipv4("1.2.3.4, 1.2.3.4/24");
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].0, "1.2.3.4".parse::<Ipv4Addr>().unwrap());
        assert_eq!(r[0].1, 32);
        assert_eq!(r[1].0, "1.2.3.4".parse::<Ipv4Addr>().unwrap());
        assert_eq!(r[1].1, 24);
    }

    #[test]
    fn test_simple_ipv6_parse() {
        let (ip, cidr) = ShapedDevice::parse_cidr_v6("fd77::1:5").unwrap();
        assert_eq!(cidr, 128);
        assert_eq!("fd77::1:5".parse::<Ipv6Addr>().unwrap(), ip);
    }

    #[test]
    fn test_cidr_ipv6_parse() {
        let (ip, cidr) = ShapedDevice::parse_cidr_v6("fd77::1:5/64").unwrap();
        assert_eq!(cidr, 64);
        assert_eq!("fd77::1:5".parse::<Ipv6Addr>().unwrap(), ip);
    }

    #[test]
    fn test_bad_ipv6_parse() {
        let r = ShapedDevice::parse_cidr_v6("bad wolf");
        assert!(r.is_err());
    }

    #[test]
    fn test_nearly_ok_ipv6_parse() {
        let r = ShapedDevice::parse_cidr_v6("fd77::1::5");
        assert!(r.is_err());
    }

    #[test]
    fn test_single_ipv6() {
        let r = ShapedDevice::parse_ipv6("fd77::1:5");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].0, "fd77::1:5".parse::<Ipv6Addr>().unwrap());
        assert_eq!(r[0].1, 128);
    }

    #[test]
    fn test_two_ipv6() {
        let r = ShapedDevice::parse_ipv6("fd77::1:5, fd77::1:5/64");
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].0, "fd77::1:5".parse::<Ipv6Addr>().unwrap());
        assert_eq!(r[0].1, 128);
        assert_eq!(r[1].0, "fd77::1:5".parse::<Ipv6Addr>().unwrap());
        assert_eq!(r[1].1, 64);
    }

    #[test]
    fn build_and_test_simple_trie() {
        let devices = vec![
            ShapedDevice{
                circuit_id: "One".to_string(),
                ipv4: ShapedDevice::parse_ipv4("192.168.1.0/24"),
                ..Default::default()
            },
            ShapedDevice{
                circuit_id: "One".to_string(),
                ipv4: ShapedDevice::parse_ipv4("1.2.3.4"),
                ..Default::default()
            },
        ];
        let trie = ConfigShapedDevices::make_trie(&devices);
        assert_eq!(trie.len(), (0, 2));
        assert!(trie.longest_match(ShapedDevice::parse_cidr_v4("192.168.2.2").unwrap().0).is_none());
        
        let addr: Ipv4Addr = "192.168.1.2".parse().unwrap();
        let v6 = addr.to_ipv6_mapped();
        assert!(trie.longest_match(v6).is_some());

        let addr: Ipv4Addr = "1.2.3.4".parse().unwrap();
        let v6 = addr.to_ipv6_mapped();
        assert!(trie.longest_match(v6).is_some());
    }
}