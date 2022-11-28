/* SPDX-License-Identifier: GPL-2.0 */
// Minimal XDP program that passes all packets.
// Used to verify XDP functionality.
#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include <linux/in6.h>
#include <linux/ip.h>
#include <linux/ipv6.h>
#include "common/debug.h"
#include "common/dissector.h"
#include "common/maximums.h"

// Constant passed in during loading to either
// 1 (facing the Internet)
// 2 (facing the LAN)
// If it stays at 255, we have a configuration error.
int direction = 255;

struct host_counter {
    __u64 download_bytes;
    __u64 upload_bytes;
    __u64 download_packets;
    __u64 upload_packets;
};

struct
{
	__uint(type, BPF_MAP_TYPE_LRU_PERCPU_HASH);
	__type(key, struct in6_addr);
	__type(value, struct host_counter);
    __uint(max_entries, MAX_TRACKED_IPS);
	__uint(pinning, LIBBPF_PIN_BY_NAME);
} map_traffic SEC(".maps");

SEC("xdp")
int xdp_prog(struct xdp_md *ctx)
{
    if (direction == 255) {
        bpf_debug("Error: interface direction unspecified, aborting.");
        return XDP_PASS;
    }
    struct dissector_t dissector = {0};
    if (!dissector_new(ctx, &dissector)) return XDP_PASS;
    if (!dissector_find_l3_offset(&dissector)) return XDP_PASS;
    if (!dissector_find_ip_header(&dissector)) return XDP_PASS;

    // Determine the lookup key by direction
    struct in6_addr key = (direction == 1) ? dissector.dst_ip : dissector.src_ip;

    // Count the bits. It's per-CPU, so we can't be interrupted - no sync required
    struct host_counter * counter = bpf_map_lookup_elem(&map_traffic, &key);
    if (counter) {
        bpf_debug("Hit map");
        if (direction == 1) {
            // Download
            counter->download_packets += 1;
            counter->download_bytes += ctx->data_end - ctx->data;
        } else {
            // Upload
            counter->download_packets += 1;
            counter->download_bytes += ctx->data_end - ctx->data;
        }
    } else {
        struct host_counter new_host = {0};
        if (direction == 1) {
            new_host.download_packets = 1;
            new_host.download_bytes = ctx->data_end - ctx->data;
        } else {
            new_host.upload_packets = 1;
            new_host.upload_bytes = ctx->data_end - ctx->data;
        }
        struct in6_addr key = (direction == 1) ? dissector.dst_ip : dissector.src_ip;
        if (bpf_map_update_elem(&map_traffic, &key, &new_host, BPF_NOEXIST) != 0) {
            bpf_debug("Failed to insert flow");
        }
    }

    bpf_debug("We've got IP. Src: %u. Dst: %u", dissector.src_ip.in6_u.u6_addr32[3], dissector.dst_ip.in6_u.u6_addr32[3]);
	return XDP_PASS;
}

char _license[] SEC("license") = "GPL";