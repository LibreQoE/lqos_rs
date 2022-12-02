#![warn(missing_docs)]

//! `lqos_sys` is a system-library that builds LibreQoS's eBPF code
//! and wraps it in a safe external Rust wrapper.
//! 
//! The `build.rs` script compiles the C code found in `src/bpf`
//! and statically embeds the result in this crate.

mod bpf_map;
mod bpf_per_cpu_map;
mod cpu_map;
mod ip_mapping;
mod kernel_wrapper;
mod lqos_kernel;
mod tcp_rtt;
mod throughput;
mod xdp_ip_address;

pub use ip_mapping::{add_ip_to_tc, clear_ips_from_tc, del_ip_from_tc, list_mapped_ips};
pub use kernel_wrapper::LibreQoSKernels;
pub use tcp_rtt::get_tcp_round_trip_times;
pub use throughput::get_throughput_map;
pub use xdp_ip_address::XdpIpAddress;
