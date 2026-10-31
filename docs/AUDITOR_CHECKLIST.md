# Auditor Checklist

Use this as the review record. Check each item only after reproducing it on the stated
commit.

## 1. State

- [ ] Check out code commit `ef13675` for release validation, or the evidence tag
      `final-audit-2026-10-31-dpa` for the full package.
- [ ] Confirm the working tree is clean.
- [ ] Read `docs/THREAT_MODEL.md`, `docs/SECURITY_CLAIMS.md`, and
      `docs/BITSLICED_MASKING.md`.
- [ ] Confirm the public label is SAL-2 locally, not SAL-4.

## 2. Reproduce the security gates

- [ ] `cargo test --workspace --all-targets --all-features --locked`
- [ ] `cargo test --workspace --doc --locked`
- [ ] `cargo +nightly-2026-08-03 clippy --workspace --all-targets --all-features --locked -- -D warnings -D clippy::pedantic -D clippy::nursery`
- [ ] `cargo +nightly-2026-08-03 miri test --workspace`
- [ ] `cargo kani --workspace`
- [ ] `RUSTFLAGS="-Zsanitizer=address" cargo +nightly-2026-08-03 test --workspace --lib --target x86_64-unknown-linux-gnu`
- [ ] `tools/run-valgrind.sh /tmp/sicas-valgrind`
- [ ] `cargo mutants --workspace --no-shuffle`
- [ ] `cargo llvm-cov --workspace --all-features --fail-under-lines 90`

Expected: zero missed viable mutants, coverage above 90% lines, and no Miri/ASan/
Valgrind errors.

## 3. Reproduce the security model checks

- [ ] `cargo run --release -p sicas-harness -- leakage-check --samples 50000`
- [ ] `python3 tools/check_stats.py`
- [ ] `python3 tools/ct_asm_check.py --matrix --lto`
- [ ] `cargo run --release -p sicas-harness -- ct --samples 1000000 --warmup 100000 --trials 3`

Expected: negative control detects correlation 1.0; masked checks stay below 0.02;
statistics match Python; assembly matrix has no branches; timing stays below `|t| > 4.5`.

## 4. Attack the new bitsliced path

- [ ] Inspect `keystream_blocks`, `add_planes`, `transpose_word`, and
      `quarter_round_bitsliced`.
- [ ] Run `cargo test -p sicas-reference bitsliced`.
- [ ] Run the `chacha20` fuzz target for at least 1,000,000 executions.
- [ ] Attempt to recover key shares from the batch output or timing.
- [ ] Check that every masked AND receives a fresh mask and that the initial state is
      independently refreshed.
- [ ] Check counter wraparound and batch/scalar interoperability.

## 5. Release and evidence

- [ ] `python3 tools/release_manifest.py --output /tmp/sicas-release`
- [ ] `python3 tools/validate_release_manifest.py --manifest /tmp/sicas-release`
      (add `--skip-commit` when validating from the evidence tag; the manifest records
      build commit `123eb69`, proof code `ef13675`)
- [ ] `tools/local-signing-dry-run.sh /tmp/sicas-release /tmp/sicas-signing`
- [ ] Run `./tools/local_proof.sh full`.
- [ ] Verify `evidence/bundle/MANIFEST.sha256`.
- [ ] Compare the release SHA-256 with
      `1f0d304ca9b9582fd9153e7c006c240bd0e06acc3a685dfad93088429e4de7da`.

## 6. Findings

- [ ] Record each finding with severity, affected claim, reproduction, and retest.
- [ ] Check the known gaps in `docs/GAP_REPORT.md`.
- [ ] Confirm whether the finding is inside the declared threat model.
- [ ] Do not treat a local pass as a SAL-4 certification.
