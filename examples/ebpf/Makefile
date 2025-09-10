# Makefile for eBPF examples
# Requires: clang, llvm, libbpf

CLANG ?= clang
LLC ?= llc
ARCH ?= x86

# Common flags
CFLAGS := -O2 -g -Wall -Wno-unused-value -Wno-pointer-sign \
          -Wno-compare-distinct-pointer-types \
          -Werror -Wno-unused-function -Wno-unused-variable

# Include paths
INCLUDES := -I/usr/include/bpf -I/usr/include/linux

# Target files
TARGETS := simple_trace.o simple_xdp.o

.PHONY: all clean

all: $(TARGETS)

%.o: %.c
	$(CLANG) $(CFLAGS) $(INCLUDES) -target bpf -c $< -o $@

clean:
	rm -f *.o

install: all
	mkdir -p ../../bin
	cp *.o ../../bin/

.PHONY: install
