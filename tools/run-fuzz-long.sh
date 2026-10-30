#!/bin/sh
# Run longer libFuzzer campaigns for the four maintained targets.
#
# Usage: tools/run-fuzz-long.sh [output-dir]
#
# FUZZ_RUNS controls masked_ops, ct_lookup, and trace_parser (default 10,000,000).
# FUZZ_CHACHA_RUNS controls chacha20 (default 1,000,000) because the masked block
# target is substantially slower.
set -eu

root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$root"
runs="${FUZZ_RUNS:-10000000}"
chacha_runs="${FUZZ_CHACHA_RUNS:-1000000}"
output="${1:-evidence/fuzz-long}"
mkdir -p "$output"

status=0
printf '# Long Fuzz Campaign\n\n' >"$output/SUMMARY.md"
printf -- '- masked_ops / ct_lookup / trace_parser runs: `%s`\n' "$runs" >>"$output/SUMMARY.md"
printf -- '- chacha20 runs: `%s`\n\n' "$chacha_runs" >>"$output/SUMMARY.md"
printf '| Target | Result |\n|---|---|\n' >>"$output/SUMMARY.md"

for target in masked_ops ct_lookup trace_parser chacha20; do
  target_runs="$runs"
  if [ "$target" = "chacha20" ]; then
    target_runs="$chacha_runs"
  fi

  if [ -f "$output/$target.log" ] \
      && grep -q "Done $target_runs runs" "$output/$target.log"; then
    printf '| %s | PASS (cached) |\n' "$target" >>"$output/SUMMARY.md"
    continue
  fi

  if cargo +nightly-2026-08-03 fuzz run "$target" -- -runs="$target_runs" \
      >"$output/$target.log" 2>&1; then
    printf '| %s | PASS |\n' "$target" >>"$output/SUMMARY.md"
  else
    printf '| %s | FAIL |\n' "$target" >>"$output/SUMMARY.md"
    status=1
  fi
done

cat "$output/SUMMARY.md"
exit "$status"
