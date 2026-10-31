#!/bin/sh
# Run the local proof gates and write a reproducible log bundle.
#
# Usage:
#   ./tools/local_proof.sh quick
#   ./tools/local_proof.sh full
#
# "quick" covers static analysis, tests, Kani, ASan, supply chain, and assembly.
# "full" adds Miri, fuzzing, mutation, coverage, cross-architecture tests, and
# release reproducibility.
set -eu

mode="${1:-quick}"
case "$mode" in
  quick|full) ;;
  *)
    echo "usage: $0 [quick|full]" >&2
    exit 2
    ;;
esac

root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$root"
stamp="$(date +%Y%m%d-%H%M%S)"
log_dir="${LOCAL_PROOF_LOG_DIR:-evidence/local-proof-$stamp}"
mkdir -p "$log_dir"
summary="$log_dir/SUMMARY.md"
status=0

run() {
  name="$1"
  shift
  printf '==> %s\n' "$name"
  if "$@" >"$log_dir/$name.log" 2>&1; then
    printf '%-28s PASS\n' "$name"
    printf '| %s | PASS |\n' "$name" >>"$summary"
  else
    printf '%-28s FAIL\n' "$name"
    printf '| %s | FAIL |\n' "$name" >>"$summary"
    status=1
    return 0
  fi
}

report() {
  name="$1"
  shift
  printf '==> %s (report-only)\n' "$name"
  if "$@" >"$log_dir/$name.log" 2>&1; then
    printf '| %s | PASS (report-only) |\n' "$name" >>"$summary"
  else
    printf '| %s | accepted misses / report-only |\n' "$name" >>"$summary"
  fi
}

printf '# Local Proof\n\n' >"$summary"
printf -- '- Mode: `%s`\n' "$mode" >>"$summary"
printf -- '- Commit: `%s`\n' "$(git rev-parse HEAD 2>/dev/null || echo unknown)" >>"$summary"
printf -- '- Generated: `%s`\n\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >>"$summary"
printf '| Gate | Result |\n|---|---|\n' >>"$summary"

run fmt cargo +nightly-2026-08-03 fmt --all -- --check
run clippy cargo +nightly-2026-08-03 clippy --workspace --all-targets --all-features --locked -- -D warnings -D clippy::pedantic -D clippy::nursery
run tests cargo test --workspace --all-targets --all-features --locked
run doctests cargo test --workspace --doc --locked
run kani cargo kani --workspace --quiet
run asan env RUSTFLAGS=-Zsanitizer=address cargo +nightly-2026-08-03 test --workspace --lib --target x86_64-unknown-linux-gnu
run audit cargo audit
run deny cargo deny check
run assembly python3 tools/ct_asm_check.py --matrix --lto --output "$log_dir/assembly.md"
run zeroize-asm python3 tools/zeroize_asm_check.py --output "$log_dir/zeroize-asm.md"
run leakage-model cargo run --quiet --release -p sicas-harness -- leakage-check --samples 50000 --threshold 0.02
run stats-differential python3 tools/check_stats.py --output "$log_dir/stats.md"
run valgrind tools/run-valgrind.sh "$log_dir/valgrind"
run release-validator-tests python3 tools/test_validate_release_manifest.py

if [ "$mode" = "full" ]; then
  run miri cargo +nightly-2026-08-03 miri test --workspace
  run mutation-core cargo mutants -p sicas-core --no-shuffle
  run mutation-reference cargo mutants -p sicas-reference --no-shuffle
  report mutation-workspace cargo mutants --workspace --no-shuffle
  run performance cargo run --quiet --release -p sicas-reference --example benchmark
  run coverage cargo llvm-cov --workspace --all-features --fail-under-lines 90 --summary-only
  run fuzz-masked cargo +nightly-2026-08-03 fuzz run masked_ops -- -runs=1000000
  run fuzz-lookup cargo +nightly-2026-08-03 fuzz run ct_lookup -- -runs=1000000
  run fuzz-trace cargo +nightly-2026-08-03 fuzz run trace_parser -- -runs=1000000
  run fuzz-chacha cargo +nightly-2026-08-03 fuzz run chacha20 -- -runs=1000000
  run fuzz-chacha-batch cargo +nightly-2026-08-03 fuzz run chacha20_batch -- -runs=1000000
  run aarch64 ./tools/run-aarch64-tests.sh
  run i686 ./tools/run-i686-tests.sh

  run release-manifest python3 tools/release_manifest.py --output "$log_dir/release"
  run release-validate python3 tools/validate_release_manifest.py --manifest "$log_dir/release"
  run signing-dry-run tools/local-signing-dry-run.sh "$log_dir/release" "$log_dir/signing"
  mkdir -p "$log_dir/repro1" "$log_dir/repro2"
  run repro1 env CARGO_TARGET_DIR="$log_dir/repro1" cargo build -p sicas-harness --release --locked
  run repro2 env CARGO_TARGET_DIR="$log_dir/repro2" cargo build -p sicas-harness --release --locked
  if cmp "$log_dir/release/sicas-harness" "$log_dir/repro1/release/sicas-harness" \
      && cmp "$log_dir/release/sicas-harness" "$log_dir/repro2/release/sicas-harness"; then
    printf '| reproducible-compare | PASS |\n' >>"$summary"
  else
    printf '| reproducible-compare | FAIL |\n' >>"$summary"
    status=1
  fi
fi

run evidence-bundle python3 tools/evidence_bundle.py --output "$log_dir/bundle"

printf '\nLogs: `%s`\n' "$log_dir" >>"$summary"
printf '==> summary: %s\n' "$summary"
exit "$status"
