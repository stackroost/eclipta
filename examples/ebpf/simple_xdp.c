// SPDX-License-Identifier: GPL-2.0
// Simple XDP eBPF program for testing

#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>

// Simple XDP program that passes all packets
SEC("xdp")
int xdp_pass(struct xdp_md *ctx)
{
    return XDP_PASS;
}

char _license[] SEC("license") = "GPL";
