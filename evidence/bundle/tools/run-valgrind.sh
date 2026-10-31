#!/bin/sh
# Run Valgrind Memcheck over the workspace library test binaries.
#
# This is a local memory-safety gate. It complements Miri and ASan; Valgrind sees
# native code and the system allocator, while Miri checks Rust semantics.
set -eu

root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$root"
output="${1:-evidence/valgrind}"
mkdir -p "$output"

cargo test --workspace --lib --no-run --locked >/dev/null

status=0
printf '# Valgrind Memcheck Evidence\n\n' >"$output/SUMMARY.md"
printf '| Binary | Result |\n|---|---|\n' >>"$output/SUMMARY.md"

for crate in sicas_core sicas_harness sicas_platform sicas_reference sicas_zeroize; do
  binary="$(find target/debug/deps -maxdepth 1 -type f -name "${crate}-*" ! -name '*.d' | sort | tail -1)"
  if [ -z "$binary" ]; then
    printf '| %s | MISSING |\n' "$crate" >>"$output/SUMMARY.md"
    status=1
    continue
  fi
  name="$(basename "$binary")"
  if valgrind \
      --error-exitcode=1 \
      --leak-check=full \
      --errors-for-leak-kinds=definite \
      "$binary" --test-threads=1 >"$output/$name.log" 2>&1; then
    printf '| %s | PASS |\n' "$crate" >>"$output/SUMMARY.md"
  else
    printf '| %s | FAIL |\n' "$crate" >>"$output/SUMMARY.md"
    status=1
  fi
done

cat "$output/SUMMARY.md"
exit "$status"
