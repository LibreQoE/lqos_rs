use crate::lqos_kernel::{
    attach_xdp_and_tc_to_interface, unload_xdp_from_interface, InterfaceDirection,
};

pub struct LibreQoSKernels {
    to_internet: String,
    to_isp: String,
}

impl LibreQoSKernels {
    pub fn new<S: ToString>(to_internet: S, to_isp: S) -> anyhow::Result<Self> {
        let kernel = Self {
            to_internet: to_internet.to_string(),
            to_isp: to_isp.to_string(),
        };
        attach_xdp_and_tc_to_interface(&kernel.to_internet, InterfaceDirection::Internet)?;
        attach_xdp_and_tc_to_interface(&kernel.to_isp, InterfaceDirection::IspNetwork)?;
        Ok(kernel)
    }
}

impl Drop for LibreQoSKernels {
    fn drop(&mut self) {
        let _ = unload_xdp_from_interface(&self.to_internet);
        let _ = unload_xdp_from_interface(&self.to_isp);
    }
}
