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
// 3 (use VLAN mode, we're running on a stick)
// If it stays at 255, we have a configuration error.
int direction = 255;
__be16 internet_vlan = 0; // Note: turn these into big-endian
__be16 isp_vlan = 0;

SEC("xdp")
int xdp_prog(struct xdp_md *ctx)
{
    if (direction == 255) {
        bpf_debug("Error: interface direction unspecified, aborting.");
        return XDP_PASS;
    }
    struct dissector_t dissector = {0};
    bpf_debug("START XDP");
    bpf_debug("Running mode %u", direction);
    bpf_debug("Scan VLANs: %u %u", internet_vlan, isp_vlan);
    if (!dissector_new(ctx, &dissector)) return XDP_PASS;
    if (!dissector_find_l3_offset(&dissector)) return XDP_PASS;
    if (!dissector_find_ip_header(&dissector)) return XDP_PASS;
    //bpf_debug("Spotted VLAN: %u", dissector.current_vlan);

    // Determine the lookup key by direction
    struct ip_hash_key lookup_key;
    int effective_direction = 0;
    struct ip_hash_info * ip_info = setup_lookup_key_and_tc_cpu(direction, &lookup_key, &dissector, internet_vlan, &effective_direction);
    bpf_debug("Effective direction: %d", effective_direction);

    __u32 tc_handle = 0;
    __u32 cpu = 0;
    if (ip_info) {
        tc_handle = ip_info->tc_handle;
        cpu = ip_info->cpu;
    }
    track_traffic(effective_direction, &lookup_key.address, ctx->data_end - ctx->data, tc_handle);

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
        bpf_debug("Zooming to CPU: %u", cpu_dest);
        bpf_debug("Mapped to handle: %u", tc_handle);
        long redirect_result = bpf_redirect_map(&cpu_map, cpu_dest, 0);
        bpf_debug("Redirect result: %u", redirect_result);
        return redirect_result;
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
    bpf_debug("SKB VLAN TCI: %u", skb->vlan_tci);    

    // Remove me
    bpf_debug("START TC");
    __u32 cpu = bpf_get_smp_processor_id();
    bpf_debug("TC egress fired on CPU %u", cpu);

    // Lookup the queue
    struct txq_config *txq_cfg;
    txq_cfg = bpf_map_lookup_elem(&map_txq_config, &cpu);
    if (!txq_cfg) return TC_ACT_SHOT;
    if (txq_cfg->queue_mapping != 0) {
		skb->queue_mapping = txq_cfg->queue_mapping;
	} else {
		bpf_debug("Misconf: CPU:%u no conf (curr qm:%d)\n", cpu, skb->queue_mapping);
	}

    // TODO: Support XDP Metadata shunt
    // In the meantime, we'll do it the hard way:
    struct tc_dissector_t dissector = {0};
    if (!tc_dissector_new(skb, &dissector)) return TC_ACT_OK;
    if (!tc_dissector_find_l3_offset(&dissector)) return TC_ACT_OK;
    if (!tc_dissector_find_ip_header(&dissector)) return TC_ACT_OK;

    // Determine the lookup key by direction
    struct ip_hash_key lookup_key;
    int effective_direction = 0;
    struct ip_hash_info * ip_info = tc_setup_lookup_key_and_tc_cpu(direction, &lookup_key, &dissector, internet_vlan, &effective_direction);
    bpf_debug("TC effective direction: %d", effective_direction);

    // Temporary pping integration - needs a lot of cleaning
    struct parsing_context context = {0};
    context.now = bpf_ktime_get_ns();
    context.skb_len = skb->len;
    context.tcp = NULL;
    context.dissector = &dissector;
    context.active_host = &lookup_key.address;
    tc_pping_start(&context);

    if (ip_info && ip_info->tc_handle != 0) {
        // We found a matching mapped TC flow
        bpf_debug("Mapped to TC handle %x", ip_info->tc_handle);
        skb->priority = ip_info->tc_handle;
        return TC_ACT_OK;
    } else {
        // We didn't find anything
        bpf_debug("TC didn't map anything");
        return TC_ACT_OK;
    }

    return TC_ACT_OK;
}

char _license[] SEC("license") = "GPL";