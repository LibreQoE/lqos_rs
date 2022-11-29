#pragma once

#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_endian.h>
#include <linux/if_ether.h>
#include <stdbool.h>
#include <linux/in6.h>
#include <linux/ip.h>
#include <linux/ipv6.h>
#include "maximums.h"
#include "debug.h"

// Data structure used for map_ip_hash
struct ip_hash_info {
	__u32 cpu;
	__u32 tc_handle; // TC handle MAJOR:MINOR combined in __u32
};

// Key type used for map_ip_hash trie
struct ip_hash_key {
	__u32 prefixlen; // Length of the prefix to match
	struct in6_addr address; // An IPv6 address. IPv4 uses the last 32 bits.
};

// Map describing IP to CPU/TC mappings
struct {
	__uint(type, BPF_MAP_TYPE_LPM_TRIE);
	__uint(max_entries, IP_HASH_ENTRIES_MAX);
	__type(key, struct ip_hash_key);
	__type(value, struct ip_hash_info);
	__uint(pinning, LIBBPF_PIN_BY_NAME);
	__uint(map_flags, BPF_F_NO_PREALLOC);
} map_ip_to_cpu_and_tc SEC(".maps");
