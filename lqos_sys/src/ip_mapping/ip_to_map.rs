use std::net::{IpAddr, Ipv6Addr, Ipv4Addr};
use anyhow::{Result, Error};

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

        if subnet_part > 128 {
            return Err(Error::msg("Invalid subnet mask"));
        }

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_ipv4_single() {
        let map = IpToMap::new("1.2.3.4", (1, 2), 1).unwrap();
        let rust_ip : IpAddr = "1.2.3.4".parse().unwrap();
        assert_eq!(rust_ip, map.subnet);
        assert_eq!(map.prefix, 128);
        assert_eq!(map.tc_handle, (1, 2));
        assert_eq!(map.cpu, 1);
    }

    #[test]
    fn parse_ipv4_subnet() {
        let map = IpToMap::new("1.2.3.0/24", (1, 2), 1).unwrap();
        let rust_ip : IpAddr = "1.2.3.0".parse().unwrap();
        assert_eq!(rust_ip, map.subnet);
        assert_eq!(map.prefix, 24+96);
        assert_eq!(map.tc_handle, (1, 2));
        assert_eq!(map.cpu, 1);
    }

    #[test]
    fn parse_ipv4_invalid_ip() {
        let map = IpToMap::new("1.2.3.256/24", (1, 2), 1);
        assert!(map.is_err());
    }

    #[test]
    fn parse_ipv4_super_invalid_ip() {
        let map = IpToMap::new("I like sheep", (1, 2), 1);
        assert!(map.is_err());
    }

    #[test]
    fn parse_ipv4_invalid_cidr() {
        let map = IpToMap::new("1.2.3.256/33", (1, 2), 1);
        assert!(map.is_err());
    }

    #[test]
    fn parse_ipv4_negative_cidr() {
        let map = IpToMap::new("1.2.3.256/-1", (1, 2), 1);
        assert!(map.is_err());
    }

    #[test]
    fn parse_ipv6_single() {
        let map = IpToMap::new("dead::beef", (1, 2), 1).unwrap();
        let rust_ip : IpAddr = "dead::beef".parse().unwrap();
        assert_eq!(rust_ip, map.subnet);
        assert_eq!(map.prefix, 128);
        assert_eq!(map.tc_handle, (1, 2));
        assert_eq!(map.cpu, 1);
    }

    #[test]
    fn parse_ipv6_subnet() {
        let map = IpToMap::new("dead:beef::/64", (1, 2), 1).unwrap();
        let rust_ip : IpAddr = "dead:beef::".parse().unwrap();
        assert_eq!(rust_ip, map.subnet);
        assert_eq!(map.prefix, 64);
        assert_eq!(map.tc_handle, (1, 2));
        assert_eq!(map.cpu, 1);
    }

    #[test]
    fn parse_ipv6_invalid_ip() {
        let map = IpToMap::new("dead:beef", (1, 2), 1);
        assert!(map.is_err());
    }
}