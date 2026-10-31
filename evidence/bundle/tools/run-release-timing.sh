#!/bin/sh
# Run release-profile dudect timing evidence and write a Markdown report.
set -eu

root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$root"
commit="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
output="${1:-evidence/ct/2026-10-31-x86_64-release.md}"
label="${2:-$commit x86_64-unknown-linux-gnu rustc 1.99.0 release}"

cargo run --release -p sicas-harness --locked -- ct \
  --samples 1000000 \
  --warmup 100000 \
  --trials 3 \
  --threshold 4.5 \
  --label "$label" \
  --report "$output"
