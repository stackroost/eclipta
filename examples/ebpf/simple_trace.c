// SPDX-License-Identifier: GPL-2.0
// Simple eBPF program for testing

#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>

// Simple program that always returns 0
SEC("tracepoint/syscalls/sys_enter_open")
int trace_open(void *ctx)
{
    return 0;
}

char _license[] SEC("license") = "GPL";
