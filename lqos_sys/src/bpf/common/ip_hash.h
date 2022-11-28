#pragma once

#include <linux/in6.h>
#include <linux/ip.h>
#include <linux/ipv6.h>

union iph_ptr
{
    struct iphdr *iph;
    struct ipv6hdr *ip6h;
};

static __always_inline void encode_ipv4(__be32 addr, struct in6_addr * out_address) {
    __builtin_memset(&out_address->in6_u.u6_addr8, 0xFF, 16);
    out_address->in6_u.u6_addr32[3] = addr;
}

static __always_inline void encode_ipv6(struct in6_addr * ipv6_address, struct in6_addr * out_address) {
    __builtin_memcpy(&out_address->in6_u.u6_addr8, &ipv6_address->in6_u.u6_addr8, 16);
}