use anyhow::Result;
use lqos_bus::{BusResponse, IpMapping};
use lqos_sys::XdpIpAddress;

fn expect_ack(result: Result<()>) -> BusResponse {
    if result.is_ok()
    {
        BusResponse::Ack
    } else {
        BusResponse::Fail(format!("{:?}", result))
    }
}

pub(crate) fn map_ip_to_flow(ip_address : &str,
    tc_major: u16,
    tc_minor: u16,
    cpu: u32) -> BusResponse
{
    expect_ack(lqos_sys::add_ip_to_tc(&ip_address, (tc_major, tc_minor), cpu))
}

pub(crate) fn del_ip_flow(ip_address : &str) -> BusResponse {
    expect_ack(lqos_sys::del_ip_from_tc(ip_address))
}

pub(crate) fn clear_ip_flows() -> BusResponse {
    expect_ack(lqos_sys::clear_ips_from_tc())
}

pub(crate) fn list_mapped_ips() -> BusResponse {
    
    if let Ok(raw) = lqos_sys::list_mapped_ips() {
        let data = raw.iter().map(|(ip_key, ip_data)| {
            IpMapping {
                ip_address: XdpIpAddress { ip: ip_key.address }.as_ip().to_string(),
                prefix_length: ip_key.prefixlen,
                tc_handle: ip_data.tc_handle,
                cpu: ip_data.cpu,
            }
        }).collect();
        BusResponse::MappedIps(data)
    } else {
        BusResponse::Fail("Unable to get IP map".to_string())
    }
}