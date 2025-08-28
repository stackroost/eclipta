// loader.c
// Build:
//   gcc -O2 -g loader.c -o ebpf_loader -lbpf -lelf -lz
//
// Run examples:
//   sudo ./ebpf_loader /path/to/program.o
//   sudo ./ebpf_loader --iface eth0 /path/to/xdp_prog.o
//   sudo ./ebpf_loader -i eth0 /path/to/xdp_prog.o
//
// This loader:
//  - auto-detects eBPF program sections (xdp, kprobe, tracepoint, uprobe, tc)
//  - loads the object into kernel (verifier)
//  - attaches programs according to detected section type
//  - keeps links alive until SIGINT/SIGTERM

#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <ifaddrs.h>
#include <net/if.h>
#include <unistd.h>
#include <getopt.h>
#include <signal.h>

#include <bpf/libbpf.h>
#include <bpf/bpf.h>
#include <linux/if_link.h>

static struct bpf_object *g_obj = NULL;
static struct bpf_link **g_links = NULL;
static int g_link_count = 0;
static int g_alloc_links = 0;
static const char *g_iface = NULL;

static void usage(const char *prog) {
    fprintf(stderr,
        "Usage: %s [--iface IFACE] <path-to-o>\n"
        "Options:\n"
        "  -i, --iface IFACE    Interface to attach XDP programs to (eg eth0)\n"
        "  -h, --help           Show this help\n\n"
        "Examples:\n"
        "  sudo %s ./xdp_pass_kern.o --iface eth0\n"
        "  sudo %s ./trace_prog.o\n",
        prog, prog, prog);
}

static int starts_with(const char *s, const char *pref) {
    if (!s || !pref) return 0;
    return strncmp(s, pref, strlen(pref)) == 0;
}

static void free_links_and_obj(void) {
    if (g_links) {
        for (int i = 0; i < g_link_count; ++i) {
            if (g_links[i]) {
                bpf_link__destroy(g_links[i]);
                g_links[i] = NULL;
            }
        }
        free(g_links);
        g_links = NULL;
    }
    if (g_obj) {
        bpf_object__close(g_obj);
        g_obj = NULL;
    }
    g_link_count = 0;
    g_alloc_links = 0;
}

static void handle_sigint(int sig) {
    (void)sig;
    fprintf(stderr, "\nReceived signal, cleaning up and detaching...\n");
    free_links_and_obj();
    exit(0);
}

int main(int argc, char **argv) {
    const char *path = NULL;

    // parse options with getopt_long
    static struct option long_options[] = {
        {"iface", required_argument, 0, 'i'},
        {"help",  no_argument,       0, 'h'},
        {0,0,0,0}
    };

    int opt;
    int option_index = 0;
    while ((opt = getopt_long(argc, argv, "i:h", long_options, &option_index)) != -1) {
        switch (opt) {
            case 'i':
                g_iface = optarg;
                break;
            case 'h':
            default:
                usage(argv[0]);
                return 0;
        }
    }

    if (optind < argc) {
        path = argv[optind];
    }

    if (!path) {
        fprintf(stderr, "Error: missing path to .o file\n");
        usage(argv[0]);
        return 1;
    }

    // register signal handlers for graceful shutdown
    struct sigaction sa;
    memset(&sa, 0, sizeof(sa));
    sa.sa_handler = handle_sigint;
    sigaction(SIGINT, &sa, NULL);
    sigaction(SIGTERM, &sa, NULL);

    struct bpf_object *obj = NULL;
    struct bpf_program *prog;
    struct bpf_link *link = NULL;
    int err;

    obj = bpf_object__open_file(path, NULL);
    if (!obj) {
        fprintf(stderr, "failed to open BPF object '%s'\n", path);
        return 1;
    }
    g_obj = obj; // for cleanup on signal

    printf("Detected program sections in %s:\n", path);
    bpf_object__for_each_program(prog, obj) {
        const char *sec = bpf_program__section_name(prog);
        printf(" - section: %s\n", sec ? sec : "(null)");
    }

    err = bpf_object__load(obj);
    if (err) {
        fprintf(stderr, "failed to load BPF object: %s\n", strerror(-err));
        bpf_object__close(obj);
        g_obj = NULL;
        return 1;
    }

    // allocate link array sized to #programs
    int prog_count = 0;
    bpf_object__for_each_program(prog, obj) prog_count++;
    if (prog_count > 0) {
        g_links = calloc(prog_count, sizeof(*g_links));
        if (!g_links) {
            fprintf(stderr, "failed to allocate link array\n");
            bpf_object__close(obj);
            g_obj = NULL;
            return 1;
        }
        g_alloc_links = prog_count;
    }

    // Attach programs based on section prefix
    bpf_object__for_each_program(prog, obj) {
        const char *sec = bpf_program__section_name(prog);
        if (!sec) sec = "";

        // Get program fd (should be valid after load)
        int prog_fd = bpf_program__fd(prog);
        if (prog_fd < 0) {
            fprintf(stderr, "warning: failed to get fd for program section %s\n", sec);
            continue;
        }

        if (starts_with(sec, "xdp")) {
            if (!g_iface) {
                printf("XDP program found (section=%s) but no --iface provided. Skipping attachment.\n", sec);
                continue;
            }
            int ifindex = if_nametoindex(g_iface);
            if (ifindex == 0) {
                fprintf(stderr, "invalid interface name '%s'\n", g_iface);
                continue;
            }

            int flags = 0; // change if you want SKB mode: XDP_FLAGS_SKB_MODE
            err = bpf_set_link_xdp_fd(ifindex, prog_fd, flags);
            if (err < 0) {
                fprintf(stderr, "failed to attach XDP program to %s: %s\n", g_iface, strerror(-err));
            } else {
                printf("Attached XDP program (section=%s) to iface %s (ifindex=%d)\n", sec, g_iface, ifindex);
                // Note: bpf_set_link_xdp_fd does not return a bpf_link object; record placeholder NULL
                // so cleanup loop knows this was attached via link-less attach (we cannot bpf_link__destroy it).
                // If you prefer to use libbpf's xdp attach helpers (returning bpf_link), swap to that API.
                // We'll still keep a placeholder in g_links to maintain indexing.
                if (g_link_count < g_alloc_links) g_links[g_link_count++] = NULL;
            }
        } else if (starts_with(sec, "kprobe") || starts_with(sec, "kretprobe") ||
                   starts_with(sec, "tracepoint") || starts_with(sec, "uprobe") ||
                   starts_with(sec, "uretprobe")) {
            link = bpf_program__attach(prog);
            if (!link) {
                fprintf(stderr, "failed to attach program section %s via libbpf\n", sec);
            } else {
                printf("Attached program section %s via libbpf (link=%p)\n", sec, (void*)link);
                if (g_link_count < g_alloc_links) g_links[g_link_count++] = link;
                else {
                    // shouldn't happen but handle gracefully
                    bpf_link__destroy(link);
                }
            }
        } else if (starts_with(sec, "tc") || starts_with(sec, "clsact") || starts_with(sec, "classifier")) {
            fprintf(stdout, "TC-like section detected (%s). TC attach not implemented by this loader.\n", sec);
            // Optionally implement TC attach logic using rtnetlink or libbpf's helper if available.
        } else {
            // Generic fallback: try libbpf attach
            link = bpf_program__attach(prog);
            if (!link) {
                fprintf(stderr, "fallback attach failed for section %s\n", sec);
            } else {
                printf("Fallback attached section %s (link=%p)\n", sec, (void*)link);
                if (g_link_count < g_alloc_links) g_links[g_link_count++] = link;
                else bpf_link__destroy(link);
            }
        }
    }

    printf("All attachments attempted. Active links stored: %d\n", g_link_count);
    printf("Loader will keep running to hold programs attached. Press Ctrl-C to exit and detach.\n");

    // Keep process alive until SIGINT/SIGTERM
    while (1) {
        pause(); // signal handler will cleanup
    }

    // unreachable, kept for completeness
    free_links_and_obj();
    return 0;
}
