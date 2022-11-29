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
#include "common/throughput.h"
#include "common/lpm.h"

// Constant passed in during loading to either
// 1 (facing the Internet)
// 2 (facing the LAN)
// If it stays at 255, we have a configuration error.
int direction = 255;

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
    struct ip_hash_key lookup_key;
    struct ip_hash_info * ip_info = setup_lookup_key_and_tc_cpu(direction, &lookup_key, &dissector);

    __u32 tc_handle = 0;
    __u32 cpu = 0;
    if (ip_info) {
        tc_handle = ip_info->tc_handle;
        cpu = ip_info->cpu;
    }
    track_traffic(direction, &lookup_key.address, ctx->data_end - ctx->data, tc_handle);
    bpf_debug("%d", cpu); // Temp

    //bpf_debug("We've got IP. Src: %u. Dst: %u", dissector.src_ip.in6_u.u6_addr32[3], dissector.dst_ip.in6_u.u6_addr32[3]);
	return XDP_PASS;
}

char _license[] SEC("license") = "GPL";