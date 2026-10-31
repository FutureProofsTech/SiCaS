#!/bin/sh
# Fast auditor smoke test: core masking, bitsliced ChaCha20, leakage model,
# statistics, assembly matrix, and a short timing run.
set -eu

root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$root"
output="${1:-/tmp/sicas-auditor-smoke}"
mkdir -p "$output"

status=0
printf '# Auditor Smoke Test\n\n' >"$output/SUMMARY.md"
printf '| Check | Result |\n|---|---|\n' >>"$output/SUMMARY.md"

run() {
  name="$1"
  shift
  printf '==> %s\n' "$name"
  if "$@" >"$output/$name.log" 2>&1; then
    printf '| %s | PASS |\n' "$name" >>"$output/SUMMARY.md"
  else
    printf '| %s | FAIL |\n' "$name" >>"$output/SUMMARY.md"
    status=1
  fi
}

run tests cargo test -p sicas-core -p sicas-reference --all-targets --all-features --locked
run leakage cargo run --release -p sicas-harness -- leakage-check --samples 50000 --threshold 0.02
run stats python3 tools/check_stats.py --output "$output/stats.md"
run assembly python3 tools/ct_asm_check.py --matrix --lto --output "$output/assembly.md"
run timing cargo run --release -p sicas-harness -- ct --samples 100000 --warmup 10000 --trials 1 --threshold 4.5

cat "$output/SUMMARY.md"
printf 'Logs: %s\n' "$output"
exit "$status"
