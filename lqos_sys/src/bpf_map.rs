use std::{ffi::{CString, c_void}, marker::PhantomData, ptr::null_mut};
use anyhow::{Result, Error};
use libbpf_sys::{bpf_obj_get, bpf_map_get_next_key, bpf_map_lookup_elem};

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

    pub fn iter(&self) -> BpfMapIterator<K,V> {
        BpfMapIterator::new(self.fd)
    }
}

impl <K,V> Drop for BpfMap<K,V> {
    fn drop(&mut self) {
        let _ = nix::unistd::close(self.fd);
    }
}

pub struct BpfMapIterator<K,V> {
    fd: i32,
    next_result: i32,
    key: K,
    prev_key : *mut K,
    current: V,
}

impl <K,V> BpfMapIterator<K,V> 
where K:Default+Clone, V:Default+Clone
{
    fn new(fd: i32) -> Self {
        // Find the first result
        let prev_key : *mut K = null_mut();
        let mut key : K = K::default();
        let key_ptr : *mut K = &mut key;
        let next_result = unsafe {
            bpf_map_get_next_key(fd, prev_key as *mut c_void, key_ptr as *mut c_void)
        };
        
        let mut current = V::default();
        if next_result != 0 {
            let current_ptr : *mut V = &mut current;
            unsafe {
                bpf_map_lookup_elem(fd, key_ptr as *mut c_void, current_ptr as *mut c_void);
            }
        }

        Self {
            fd,
            next_result,
            key,
            current,
            prev_key,
        }
    }
}

impl <K,V> Iterator for BpfMapIterator<K, V> 
where K:Default+Clone, V:Default+Clone
{
    type Item = (K,V);

    fn next(&mut self) -> Option<Self::Item> {
        // Return if no data
        if self.next_result != 0 {
            return None;
        }

        let key_ptr : *mut K = &mut self.key;
        let key = self.key.clone();
        let value = self.current.clone();

        self.prev_key = &mut self.key;
        self.next_result = unsafe {
            bpf_map_get_next_key(self.fd, self.prev_key as *mut c_void, key_ptr as *mut c_void)
        };
        if self.next_result != 0 {
            let current_ptr : *mut V = &mut self.current;
            unsafe {
                bpf_map_lookup_elem(self.fd, key_ptr as *mut c_void, current_ptr as *mut c_void);
            }
        }

        Some((key.clone(), value.clone()))
    }
}