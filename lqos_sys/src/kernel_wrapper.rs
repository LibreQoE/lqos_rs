use crate::lqos_kernel::{
    attach_xdp_and_tc_to_interface, unload_xdp_from_interface, InterfaceDirection,
};

/// A wrapper-type that stores the interfaces to which the XDP and TC programs should
/// be attached. Performs the attachment process, and hooks "drop" to unattach the
/// programs when the structure falls out of scope.
pub struct LibreQoSKernels {
    to_internet: String,
    to_isp: String,
}

impl LibreQoSKernels {
    /// Create a new `LibreQosKernels` structure, using the specified interfaces.
    /// Returns Ok(self) if attaching to the XDP/TC interfaces succeeded, otherwise
    /// returns an error containing a string describing what went wrong.
    /// 
    /// Outputs progress to `stdio` during execution, and detailed errors to `stderr`.
    /// 
    /// ## Arguments
    /// 
    /// * `to_internet` - the name of the Internet-facing interface (e.g. `eth1`).
    /// * `to_isp` - the name of the ISP-network facing interface (e.g. `eth2`).
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
