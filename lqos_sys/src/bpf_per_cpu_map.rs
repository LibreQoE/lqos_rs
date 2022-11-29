use std::{ffi::{CString, c_void}, marker::PhantomData, ptr::null_mut};
use std::fmt::Debug;
use anyhow::{Result, Error};
use libbpf_sys::{bpf_obj_get, bpf_map_get_next_key, bpf_map_lookup_elem, libbpf_num_possible_cpus};

pub struct BpfPerCpuMap<K, V> {
    fd: i32,
    _key_phantom: PhantomData<K>,
    _val_phantom: PhantomData<V>,
}

impl<K,V> BpfPerCpuMap<K,V> 
where K:Default+Clone, V:Default+Clone+Debug
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

    pub fn iter(&self) -> BpfPerCpuMapIterator<K,V> {
        let num_cpus = unsafe {
            libbpf_num_possible_cpus()
        };
        BpfPerCpuMapIterator::new(self.fd, num_cpus as usize)
    }
}

impl <K,V> Drop for BpfPerCpuMap<K,V> {
    fn drop(&mut self) {
        let _ = nix::unistd::close(self.fd);
    }
}

pub struct BpfPerCpuMapIterator<K,V> {
    fd: i32,
    num_cpus: usize,
    next_result: Option<(usize, K, Vec<V>)>,
    value_store: Vec<V>, // To avoid reallocation
    prev_key : *mut K,
    key: K,
}

impl <K,V> BpfPerCpuMapIterator<K,V> 
where K:Default+Clone, V:Default+Clone+Debug
{
    fn new(fd: i32, num_cpus: usize) -> Self {
        // Find the first result
        let mut next_result = None;
        let prev_key : *mut K = null_mut();
        let mut key : K = K::default();
        let key_ptr : *mut K = &mut key;
        let err = unsafe {
            bpf_map_get_next_key(fd, prev_key as *mut c_void, key_ptr as *mut c_void)
        };
        let mut value_store = vec![V::default(); num_cpus];
        if err == 0 {
            // We found some data, so set the next result
            let value_store_ptr : *mut V = value_store.as_mut_ptr();
            unsafe {
                bpf_map_lookup_elem(fd, key_ptr as *mut c_void, value_store_ptr as *mut c_void);
            }
            next_result = Some((0, key.clone(), value_store.clone()));
        }

        Self {
            fd,
            num_cpus,
            next_result,
            value_store,
            prev_key,
            key,
        }
    }
}

impl <K,V> Iterator for BpfPerCpuMapIterator<K, V> 
where K:Default+Clone, V:Default+Clone+Debug
{
    type Item = (K,V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((count, key, value)) = &mut self.next_result {
            if *count < self.num_cpus {
                *count += 1;
                return Some((key.clone(), value[*count-1].clone()));
            }
        } else {
            // No Data - Return
            return None;
        }

        let key_ptr : *mut K = &mut self.key;
        let err = unsafe {
            bpf_map_get_next_key(self.fd, self.prev_key as *mut c_void, key_ptr as *mut c_void)
        };
        if err != 0 {
            // No more data
            self.next_result = None;
        } else {
            // There's more
            self.prev_key = &mut self.key;
            let current_ptr : *mut V = self.value_store.as_mut_ptr();
            unsafe {
                bpf_map_lookup_elem(self.fd, key_ptr as *mut c_void, current_ptr as *mut c_void);
            }
            self.next_result = Some((1, self.key.clone(), self.value_store.clone()));
        }

        Some((self.key.clone(), self.value_store[0].clone()))
    }
}