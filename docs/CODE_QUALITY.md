# Code Quality Standard

The project standard is: every line is reviewable, every public item is documented,
every security property has a proof obligation, and every proof obligation has a tool
and a published result.

## Definition of done

Every module and function must have:

- public API documentation with a safety/leakage contract and tested examples;
- unit tests for nominal, boundary, invalid-input, and error paths;
- property or metamorphic tests for algebraic invariants;
- compile-fail tests for forbidden API use;
- a fuzz target for every parser or byte-oriented input;
- Miri and applicable sanitizers;
- Kani/model checking for critical invariants where tractable;
- assembly inspection for constant-time and security-critical code;
- mutation-testing coverage and a reviewed coverage report;
- two-person review, with independent review for security-critical changes;
- no `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, or `dbg!` in production
  paths;
- no secret-dependent branch or index;
- no secret in logs, errors, `Debug`, or panic messages;
- no silent fallback: security failures must fail closed.

## Lint policy

The workspace denies:

- all warnings;
- `clippy::pedantic` and `clippy::nursery`;
- `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `dbg_macro`;
- `indexing_slicing`, `arithmetic_side_effects`, `integer_division`, `as_conversions`;
- `missing_docs`, `missing_debug_implementations`, `unreachable_pub`, `unused_must_use`;
- broken, missing, and private intra-doc links.

Test-only code may allow a restriction lint locally with a written reason. Blanket
allows are not accepted.

## Required commands

```bash
cargo +nightly-2026-08-03 fmt --all -- --check

cargo +nightly-2026-08-03 clippy \
  --workspace --all-targets --all-features --locked \
  -- -D warnings -D clippy::pedantic -D clippy::nursery

cargo test --workspace --all-targets --all-features --locked
cargo test --workspace --doc --locked
cargo +nightly-2026-08-03 miri test --workspace
cargo kani --workspace
RUSTFLAGS="-Zsanitizer=address" cargo +nightly-2026-08-03 test --workspace --lib \
  --target x86_64-unknown-linux-gnu
cargo +nightly-2026-08-03 fuzz run masked_ops -- -runs=10000
cargo +nightly-2026-08-03 fuzz run ct_lookup -- -runs=10000
cargo +nightly-2026-08-03 fuzz run trace_parser -- -runs=10000
cargo +nightly-2026-08-03 fuzz run chacha20 -- -runs=10000
cargo mutants --workspace --no-shuffle
cargo audit
cargo deny check
cargo llvm-cov --workspace --all-features --fail-under-lines 90
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --locked

# Release manifest, checksums, SBOM, provenance
python3 tools/release_manifest.py --output release
python3 tools/validate_release_manifest.py --manifest release
python3 tools/test_validate_release_manifest.py

# First-order leakage-model check with a negative control
cargo run --release -p sicas-harness -- leakage-check --samples 50000

# Assembly matrix and Valgrind
python3 tools/ct_asm_check.py --matrix --lto
tools/run-valgrind.sh evidence/valgrind

# Differential statistics and local signing dry run
python3 tools/check_stats.py
tools/local-signing-dry-run.sh release evidence/signing
```

## Dependency policy

- No third-party runtime dependencies.
- No build dependencies in published crates.
- Dev-only verification tools live in non-published members or run as external
  binaries.
- `cargo tree --workspace --edges normal` must show only workspace crates.

## Review policy

- Security-critical changes require an independent reviewer.
- Every review checks the leakage contract, the proof obligation, and the evidence.
- A change that weakens a claim downgrades the claim until new evidence exists.
