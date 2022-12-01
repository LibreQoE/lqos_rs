#![allow(dead_code)]
use std::{ffi::{CString, c_void}, marker::PhantomData, ptr::null_mut};
use anyhow::{Result, Error};
use libbpf_sys::{bpf_obj_get, bpf_map_get_next_key, bpf_map_lookup_elem, bpf_map_update_elem, BPF_NOEXIST};

pub struct BpfMap<K, V> {
    fd: i32,
    _key_phantom: PhantomData<K>,
    _val_phantom: PhantomData<V>,
}

impl<K,V> BpfMap<K,V> 
where K:Default+Clone, V:Default+Clone
{
    pub fn from_path(filename: &str) -> Result<Self> {
        let filename_c = CString::new(filename)?;
        let fd = unsafe {
            bpf_obj_get(filename_c.as_ptr())
        };
        if fd < 0 {
            Err(Error::msg("Unable to open BPF map"))
        } else {
            Ok(
                Self {
                    fd,
                    _key_phantom: PhantomData,
                    _val_phantom: PhantomData,
                }
            )
        }
    }

    pub fn dump_vec(&self) -> Vec<(K, V)> {
        let mut result = Vec::new();

        let mut prev_key : *mut K = null_mut();
        let mut key : K = K::default();
        let key_ptr : *mut K = &mut key;
        let mut value = V::default();
        let value_ptr : *mut V = &mut value;

        unsafe {
            while bpf_map_get_next_key(self.fd, prev_key as *mut c_void, key_ptr as *mut c_void) == 0 {
                bpf_map_lookup_elem(self.fd, key_ptr as *mut c_void, value_ptr as *mut c_void);
                result.push((
                    key.clone(),
                    value.clone(),
                ));
                prev_key = key_ptr;
            }
        }

        result
    }

    pub fn insert(&mut self, key: &mut K, value: &mut V) -> Result<()> {
        let key_ptr : *mut K = key;
        let val_ptr : *mut V = value;
        let err = unsafe {
            bpf_map_update_elem(self.fd, key_ptr as *mut c_void, val_ptr as *mut c_void, BPF_NOEXIST.into())
        };
        if err != 0 {
            Err(Error::msg("Unable to insert into map"))
        } else {
            Ok(())
        }
    }
}

impl <K,V> Drop for BpfMap<K,V> {
    fn drop(&mut self) {
        let _ = nix::unistd::close(self.fd);
    }
}
