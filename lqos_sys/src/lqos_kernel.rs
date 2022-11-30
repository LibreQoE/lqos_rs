#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use anyhow::{Error, Result};
use libbpf_sys::{
    bpf_xdp_attach, libbpf_set_strict_mode, LIBBPF_STRICT_ALL,
    XDP_FLAGS_UPDATE_IF_NOEXIST,
};
use nix::libc::{if_nametoindex, geteuid};
use std::{ffi::{CString}};
use crate::cpu_map::CpuMapping;

mod bpf {
    #![allow(warnings, unused)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub fn check_root() -> Result<()> {
    unsafe {
        if geteuid()==0 {
            Ok(())
        } else {
            Err(Error::msg("You need to be root to do this."))
        }
    }
}

pub fn interface_name_to_index(interface_name: &str) -> Result<u32> {
    let if_name = CString::new(interface_name)?;
    let index = unsafe { if_nametoindex(if_name.as_ptr()) };
    if index == 0 {
        Err(Error::msg("Unknown interface: {interface_name"))
    } else {
        Ok(index)
    }
}

pub fn unload_xdp_from_interface(interface_name: &str) -> Result<()> {
    println!("Unloading XDP/TC");
    check_root()?;
    let interface_index = interface_name_to_index(interface_name)?.try_into()?;
    unsafe {
        let err = bpf_xdp_attach(
            interface_index,
            -1,
            1 << 0,
            std::ptr::null(),
        );
        if err != 0 {
            return Err(Error::msg("Unable to unload from interface."));
        }

        let interface_c = CString::new(interface_name)?;
        let _ = bpf::tc_detach_egress(interface_index as i32, true, true, interface_c.as_ptr());
    }
    Ok(())
}

fn set_strict_mode() -> Result<()> {
    let err = unsafe { libbpf_set_strict_mode(LIBBPF_STRICT_ALL) };
    if err != 0 {
        Err(Error::msg("Unable to activate BPF Strict Mode"))
    } else {
        Ok(())
    }
}

unsafe fn open_kernel() -> Result<*mut bpf::lqos_kern> {
    let result = bpf::lqos_kern_open();
    if result.is_null() {
        Err(Error::msg("Unable to open LibreQoS XDP/TC Kernel"))
    } else {
        Ok(result)
    }
}

unsafe fn load_kernel(skeleton: *mut bpf::lqos_kern) -> Result<()> {
    let error = bpf::lqos_kern_load(skeleton);
    if error != 0 {
        Err(Error::msg("Unable to load the XDP/TC kernel"))
    } else {
        Ok(())
    }
}

#[derive(PartialEq, Eq)]
pub enum InterfaceDirection {
    Internet,
    IspNetwork,
}

pub fn attach_xdp_and_tc_to_interface(interface_name: &str, direction: InterfaceDirection) -> Result<()> {
    check_root()?;
    // Check the interface is valid
    let interface_index = interface_name_to_index(interface_name)?;
    set_strict_mode()?;
    let skeleton = unsafe {
        let skeleton = open_kernel()?;
        (*(*skeleton).data).direction = match direction {
            InterfaceDirection::Internet => 1,
            InterfaceDirection::IspNetwork => 2,
        };
        load_kernel(skeleton)?;
        let _ = unload_xdp_from_interface(interface_name); // Ignoring error, it's ok if there isn't one
        let prog_fd = bpf::bpf_program__fd((*skeleton).progs.xdp_prog);
        let error = bpf_xdp_attach(
            interface_index.try_into().unwrap(),
            prog_fd,
            XDP_FLAGS_UPDATE_IF_NOEXIST,
            std::ptr::null(),
        );
        if error != 0 {
            return Err(Error::msg("Unable to attach to interface"));
        }
        skeleton
    };

    // Configure CPU Maps - only do this for the Internet direction
    if direction == InterfaceDirection::Internet {
        let cpu_map = CpuMapping::new()?;
        cpu_map.mark_cpus_available()?;
    }

    // Attach the TC program
    // extern int tc_attach_egress(int ifindex, bool verbose, struct lqos_kern *obj);
    // extern int tc_detach_egress(int ifindex, bool verbose, bool flush_hook, char * ifname);
    let interface_c = CString::new(interface_name)?;
    let _ = unsafe {
        bpf::tc_detach_egress(interface_index as i32, true, true, interface_c.as_ptr())
    }; // Ignoring error, because it's ok to not have something to detach

    let error = unsafe {
        bpf::tc_attach_egress(interface_index as i32, true, skeleton)
    };
    if error != 0 {
        return Err(Error::msg("Unable to attach TC to interface"));
    }

    Ok(())
}
