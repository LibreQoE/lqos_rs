mod lqos_kernel;

pub use lqos_kernel::unload_xdp_from_interface;
pub use lqos_kernel::attach_xdp_to_interface;
pub use lqos_kernel::InterfaceDirection;
pub use lqos_kernel::BpfMapReader;
