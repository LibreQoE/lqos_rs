/* SPDX-License-Identifier: GPL-2.0 */
// Minimal XDP program that passes all packets.
// Used to verify XDP functionality.
#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include "common/debug.h"
#include "common/dissector.h"

// Constant passed in during loading to either
// 1 (arriving from the Internet)
// 2 (arriving from the LAN)
// If it stays at 255, we have a configuration error.
int direction = 255;

SEC("xdp")
int  xdp_prog(struct xdp_md *ctx)
{
    if (direction == 255) {
        bpf_debug("Error: did not set direction");
    } else {
        bpf_debug("Direction = %d", direction);
    }
    struct dissector_t dissector = {0};
    if (!dissector_new(ctx, &dissector)) return XDP_PASS;
    if (!dissector_find_l3_offset(&dissector)) return XDP_PASS;
    if (!dissector_find_ip_header(&dissector)) return XDP_PASS;
    bpf_debug("We've got IP. Src: %u. Dst: %u", dissector.src_ip.in6_u.u6_addr32[3], dissector.dst_ip.in6_u.u6_addr32[3]);
	return XDP_PASS;
}

char _license[] SEC("license") = "GPL";