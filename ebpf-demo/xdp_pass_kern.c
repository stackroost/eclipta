// xdp_pass_kern.c
#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>

// XDP program: just passes all packets without dropping
SEC("xdp")
int xdp_pass(struct xdp_md *ctx) {
    return XDP_PASS; // let packets continue
}

// Required license declaration
char LICENSE[] SEC("license") = "GPL";
