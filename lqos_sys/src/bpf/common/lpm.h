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
#include "dissector.h"
#include "dissector_tc.h"

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

// RECIPROCAL Map describing IP to CPU/TC mappings
// If in "on a stick" mode, this is used to
// fetch the UPLOAD mapping.
struct {
	__uint(type, BPF_MAP_TYPE_LPM_TRIE);
	__uint(max_entries, IP_HASH_ENTRIES_MAX);
	__type(key, struct ip_hash_key);
	__type(value, struct ip_hash_info);
	__uint(pinning, LIBBPF_PIN_BY_NAME);
	__uint(map_flags, BPF_F_NO_PREALLOC);
} map_ip_to_cpu_and_tc_recip SEC(".maps");

static __always_inline struct ip_hash_info * setup_lookup_key_and_tc_cpu(
    int direction, 
    struct ip_hash_key * lookup_key, 
    struct dissector_t * dissector,
    __be16 internet_vlan,
    int * out_effective_direction
) 
{
    lookup_key->prefixlen = 128;
    if (direction < 3) {
        lookup_key->address = (direction == 1) ? dissector->dst_ip : dissector->src_ip;
        *out_effective_direction = direction;
        struct ip_hash_info * ip_info = bpf_map_lookup_elem(&map_ip_to_cpu_and_tc, lookup_key);
        return ip_info;
    } else {
        if (dissector->current_vlan == internet_vlan) {
            // Packet is coming IN from the Internet.
            // Therefore it is download.
            lookup_key->address = dissector->dst_ip;
            *out_effective_direction = 1;
            struct ip_hash_info * ip_info = bpf_map_lookup_elem(&map_ip_to_cpu_and_tc, lookup_key);
            return ip_info;
        } else {
            // Packet is coming IN from the ISP.
            // Therefore it is UPLOAD.
            lookup_key->address = dissector->src_ip;
            *out_effective_direction = 2;
            struct ip_hash_info * ip_info = bpf_map_lookup_elem(&map_ip_to_cpu_and_tc_recip, lookup_key);
            return ip_info;
        }
    }
}

static __always_inline struct ip_hash_info * tc_setup_lookup_key_and_tc_cpu(
    int direction, 
    struct ip_hash_key * lookup_key, 
    struct tc_dissector_t * dissector,
    __be16 internet_vlan,
    int * out_effective_direction
) 
{
    lookup_key->prefixlen = 128;
	// Direction is reversed because we are operating on egress
    if (direction < 3) {
        lookup_key->address = (direction == 1) ? dissector->src_ip : dissector->dst_ip;
        *out_effective_direction = direction;
        struct ip_hash_info * ip_info = bpf_map_lookup_elem(&map_ip_to_cpu_and_tc, lookup_key);
        return ip_info;
    } else {
        if (dissector->current_vlan == internet_vlan) {
            // Packet is going OUT to the Internet.
            // Therefore, it is UPLOAD.
            lookup_key->address = dissector->src_ip;
            *out_effective_direction = 2;
            struct ip_hash_info * ip_info = bpf_map_lookup_elem(&map_ip_to_cpu_and_tc_recip, lookup_key);
            return ip_info;
        } else {
            // Packet is going OUT to the LAN.
            // Therefore, it is DOWNLOAD.
            lookup_key->address = dissector->dst_ip;
            *out_effective_direction = 1;
            lookup_key->address = (dissector->current_vlan == internet_vlan) ? dissector->src_ip : dissector->dst_ip;
            *out_effective_direction = (dissector->current_vlan == internet_vlan) ? 2 : 1;
        }
    }
    struct ip_hash_info * ip_info = bpf_map_lookup_elem(&map_ip_to_cpu_and_tc, lookup_key);
    return ip_info;
}