#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="/eclipta"
BIN_DIR="$ROOT_DIR/bin"
PROJ_DIR="$ROOT_DIR/ebpf-demo"

cd "$PROJ_DIR"

rustup toolchain install nightly --profile minimal || true
rustup target add bpfel-unknown-none --toolchain nightly || true

if ! command -v bpf-linker >/dev/null 2>&1; then
  cargo +nightly install bpf-linker --locked
fi

cargo +nightly build -Z build-std=core --target bpfel-unknown-none -p ebpf --release

OUT_DIR="$PROJ_DIR/target/bpfel-unknown-none/release"
OBJ=""

if [[ -f "$OUT_DIR/libebpf.so" ]]; then
  OBJ="$OUT_DIR/libebpf.so"
else
  # Fallback to the newest non-bitcode .o in release (exclude deps/*.o which are bitcode)
  CANDIDATE=$(find "$OUT_DIR" -maxdepth 1 -type f -name "*.o" -printf "%T@ %p\n" | sort -nr | head -n1 | awk '{print $2}')
  if [[ -n "${CANDIDATE:-}" ]] && file "$CANDIDATE" | grep -q "ELF"; then
    OBJ="$CANDIDATE"
  fi
fi

if [[ -z "${OBJ:-}" ]]; then
  echo "No ELF eBPF object found under $OUT_DIR" >&2
  exit 1
fi

mkdir -p "$BIN_DIR"
cp -f "$OBJ" "$BIN_DIR/ebpf.so"
echo "Built and moved: $BIN_DIR/ebpf.so" 