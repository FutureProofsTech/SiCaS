#!/bin/sh
# Run the workspace library tests for aarch64 Linux under QEMU user mode.
#
# This is functional cross-architecture evidence, not timing or hardware
# leakage evidence. Override AARCH64_SYSROOT for non-Fedora distributions.
set -eu

sysroot="${AARCH64_SYSROOT:-/usr/aarch64-redhat-linux/sys-root/fc44}"
if [ ! -d "$sysroot" ]; then
  echo "AArch64 sysroot not found: $sysroot" >&2
  exit 1
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT INT TERM

mkdir -p "$work/sysroot" "$work/lib"
ln -s "$sysroot/usr/lib" "$work/sysroot/lib"
ln -s "$sysroot/usr/lib64" "$work/sysroot/lib64"

libgcc="$(aarch64-linux-gnu-gcc -print-libgcc-file-name)"
if [ ! -f "$libgcc" ]; then
  echo "AArch64 libgcc not found: $libgcc" >&2
  exit 1
fi
cp "$libgcc" "$work/lib/libgcc_s.a"

cat > "$work/linker" <<EOF
#!/bin/sh
exec aarch64-linux-gnu-gcc --sysroot="$sysroot" -L"$work/lib" "\$@"
EOF
chmod +x "$work/linker"

CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="$work/linker" \
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER="qemu-aarch64 -L $work/sysroot" \
cargo test --workspace --lib --target aarch64-unknown-linux-gnu --locked
