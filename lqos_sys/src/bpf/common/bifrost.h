#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_endian.h>
#include <linux/if_ether.h>
#include <stdbool.h>
#include "maximums.h"
#include "debug.h"

struct bifrost_interface {
    __u32 redirect_to;
    __u32 direction;
};

struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, 64);
	__type(key, __u32);
	__type(value, struct bifrost_interface);
	__uint(pinning, LIBBPF_PIN_BY_NAME);
} bifrost_interface_map SEC(".maps");

struct bifrost_vlan {
    __u16 match_tag;
    __u16 new_tag;
    __u32 direction;
    __u32 interface;
};

struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, 64);
	__type(key, __u32);
	__type(value, struct bifrost_vlan);
	__uint(pinning, LIBBPF_PIN_BY_NAME);
} bifrost_vlan_map SEC(".maps");