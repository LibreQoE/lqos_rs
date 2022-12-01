/* SPDX-License-Identifier: GPL-2.0 */
// Minimal XDP program that passes all packets.
// Used to verify XDP functionality.
#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include <linux/in6.h>
#include <linux/ip.h>
#include <linux/ipv6.h>
#include <linux/pkt_cls.h>
#include <linux/pkt_sched.h> /* TC_H_MAJ + TC_H_MIN */
#include "common/debug.h"
#include "common/dissector.h"
#include "common/dissector_tc.h"
#include "common/maximums.h"
#include "common/throughput.h"
#include "common/lpm.h"
#include "common/cpu_map.h"
#include "common/tcp_rtt.h"

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

    // Send on its way
    if (tc_handle != 0) {
        // Handle CPU redirection if there is one specified
        __u32 *cpu_lookup;
        cpu_lookup = bpf_map_lookup_elem(&cpus_available, &cpu);
        if (!cpu_lookup) {
            bpf_debug("Error: CPU %u is not mapped", cpu);
            return XDP_PASS; // No CPU found
        }
        __u32 cpu_dest = *cpu_lookup;

        // Redirect based on CPU
        //bpf_debug("Zooming to CPU: %u", cpu_dest);
        return bpf_redirect_map(&cpu_map, cpu_dest, 0); 
    }
	return XDP_PASS;
}

SEC("tc")
int tc_iphash_to_cpu(struct __sk_buff *skb)
{
    if (direction == 255) {
        bpf_debug("Error: interface direction unspecified, aborting.");
        return TC_ACT_OK;
    }

    // Remove me
    __u32 cpu = bpf_get_smp_processor_id();
    bpf_debug("TC egress fired on CPU %u", cpu);

    // TODO: Support XDP Metadata shunt
    // In the meantime, we'll do it the hard way:
    struct tc_dissector_t dissector = {0};
    if (!tc_dissector_new(skb, &dissector)) return TC_ACT_OK;
    if (!tc_dissector_find_l3_offset(&dissector)) return TC_ACT_OK;
    if (!tc_dissector_find_ip_header(&dissector)) return TC_ACT_OK;

    // Determine the lookup key by direction
    struct ip_hash_key lookup_key;
    struct ip_hash_info * ip_info = tc_setup_lookup_key_and_tc_cpu(direction, &lookup_key, &dissector);

    __u32 tc_handle = 0;
    if (ip_info) tc_handle = ip_info->tc_handle;

    // Temporary pping integration - needs a lot of cleaning
    struct parsing_context context = {0};
    context.now = bpf_ktime_get_ns();
    context.data = (void *)(long)skb->data;
    context.data_end = (void *)(long)skb->data_end;
    context.ip_header = dissector.ip_header;
    context.l3_offset = dissector.l3offset;
    context.now = 0;
    context.protocol = dissector.eth_type;
    context.skb_len = skb->len;
    context.tc_handle = tc_handle;
    context.tcp = NULL;
    tc_pping_start(&context);

    if (ip_info && ip_info->tc_handle != 0) {
        // We found a matching mapped TC flow
        bpf_debug("Mapped to TC handle %u", ip_info->tc_handle);
        skb->priority = ip_info->tc_handle;
    } else {
        // We didn't find anything
        return TC_ACT_OK;
    }

    return TC_ACT_OK;
}

char _license[] SEC("license") = "GPL";