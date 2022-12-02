use std::{ffi::CString, os::raw::c_void};
use anyhow::{Error, Result};
use libbpf_sys::{bpf_map_update_elem, bpf_obj_get, libbpf_num_possible_cpus};

//* Provides an interface for querying the number of CPUs eBPF can
//* see, and marking CPUs as available. Currently marks ALL eBPF
//* usable CPUs as available.

pub(crate) struct CpuMapping {
    fd_cpu_map: i32,
    fd_cpu_available: i32,
}

fn get_map_fd(filename: &str) -> Result<i32> {
    let filename_c = CString::new(filename)?;
    let fd = unsafe { bpf_obj_get(filename_c.as_ptr()) };
    if fd < 0 {
        Err(Error::msg("Unable to open BPF map"))
    } else {
        Ok(fd)
    }
}

impl CpuMapping {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            fd_cpu_map: get_map_fd("/sys/fs/bpf/cpu_map")?,
            fd_cpu_available: get_map_fd("/sys/fs/bpf/cpus_available")?,
        })
    }

    pub(crate) fn mark_cpus_available(&self) -> Result<()> {
        let cpu_count = unsafe { libbpf_num_possible_cpus() } as u32;

        let queue_size = 2048u32;
        let val_ptr: *const u32 = &queue_size;
        for cpu in 0..cpu_count {
            println!("Mapping core #{cpu}");
            // Insert into the cpu map
            let cpu_ptr: *const u32 = &cpu;
            let error = unsafe {
                bpf_map_update_elem(
                    self.fd_cpu_map,
                    cpu_ptr as *const c_void,
                    val_ptr as *const c_void,
                    0,
                )
            };
            if error != 0 {
                return Err(Error::msg("Unable to map CPU"));
            }

            // Insert into the available list
            let error = unsafe {
                bpf_map_update_elem(
                    self.fd_cpu_available,
                    cpu_ptr as *const c_void,
                    cpu_ptr as *const c_void,
                    0,
                )
            };
            if error != 0 {
                return Err(Error::msg("Unable to add to available CPUs list"));
            }
        } // CPU loop
        Ok(())
    }
}

impl Drop for CpuMapping {
    fn drop(&mut self) {
        let _ = nix::unistd::close(self.fd_cpu_available);
        let _ = nix::unistd::close(self.fd_cpu_map);
    }
}
