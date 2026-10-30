# SiCaS

Safe, zero-dependency Rust primitives for side-channel attenuation and leakage
assessment.

## Status

Early Phase 1. The core crate is not audited, not hardware-validated, and **must not
be used to protect real secrets yet**. The production target is SAL-4 as defined in
[`PLAN.md`](PLAN.md) and [`docs/ASSURANCE.md`](docs/ASSURANCE.md).

## What exists

- `sicas-core`: `no_std`, `#![forbid(unsafe_code)]`, zero runtime dependencies.
  - `Choice` and constant-time comparison/selection/swap/lookup for
    `u8`, `u16`, `u32`, `u64`, and `u128`.
  - `Secret<T>` with redacted `Debug` and no accidental equality/indexing/cloning.
  - `Masked<T>` with two-share first-order Boolean masking for
    `u8`, `u16`, `u32`, `u64`, and `u128`.
  - `EntropySource` with fail-closed behavior.
- `sicas-harness`: `std`, zero third-party dependencies.
  - Strict binary trace format with typed errors and a fuzzed parser.
  - Trace preprocessing: trim, moving-average filter, and cross-correlation alignment.
  - Fixed-vs-random TVLA with Welch's t-test.
  - Byte-oriented CPA with Hamming-weight and custom leakage models.
  - Deterministic synthetic trace generation.
  - Dudect-style timing tooling.
  - CLI: `tvla`, `cpa`, `synthetic`, `synthetic-cpa`, `timing`.
- `sicas-reference`: `no_std`, zero third-party dependencies.
  - First-order Boolean masked ChaCha20 keystream blocks.
  - RFC 8439 vector and differential tests against an unmasked reference.
- `sicas-platform`: safe, zero third-party dependencies.
  - Fail-closed OS entropy adapter for Unix-like systems.
  - Unsupported platforms return `EntropyError::Unavailable`.
- Unit tests, exhaustive byte tests, property-style randomized tests, compile-fail
  doctests, Kani proof harnesses, and libFuzzer targets.
- AddressSanitizer, Miri, strict formatting, and strict Clippy gates.
- Dudect-style constant-time evidence suite and assembly inspection target.

See:

- [`docs/EVALUATION.md`](docs/EVALUATION.md) — trace format and harness usage
- [`docs/HARDWARE.md`](docs/HARDWARE.md) — hardware evidence protocol
- [`docs/CT_EVIDENCE.md`](docs/CT_EVIDENCE.md) — constant-time timing evidence
- [`docs/FUZZ_EVIDENCE.md`](docs/FUZZ_EVIDENCE.md) — 1M-execution fuzz campaigns
- [`docs/RELEASE.md`](docs/RELEASE.md) — release, SBOM, and signing
- [`docs/RED_TEAM.md`](docs/RED_TEAM.md) — independent review program
- [`docs/AUDIT_PACKAGE.md`](docs/AUDIT_PACKAGE.md) — audit scope and evidence index
- [`docs/GAP_REPORT.md`](docs/GAP_REPORT.md) — remaining external gates

## Quality gates

```bash
cargo +nightly-2026-08-03 fmt --all -- --check

cargo +nightly-2026-08-03 clippy \
  --workspace --all-targets --all-features --locked \
  -- -D warnings -D clippy::pedantic -D clippy::nursery

cargo test --workspace --all-targets --all-features --locked
cargo test --workspace --doc --locked
cargo +nightly-2026-08-03 miri test --workspace
cargo kani
RUSTFLAGS="-Zsanitizer=address" cargo +nightly-2026-08-03 test -p sicas-core --lib
cargo +nightly-2026-08-03 fuzz run masked_ops -- -runs=10000
cargo +nightly-2026-08-03 fuzz run ct_lookup -- -runs=10000
cargo mutants --workspace --no-shuffle
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --locked
```

## Constraints

- Safe Rust: `sicas-core` forbids unsafe code.
- Zero runtime dependencies: no third-party crates.
- The non-published `fuzz/` crate uses `libfuzzer-sys`; it is excluded from the workspace
  and is never shipped.
- Every security claim must link to reproducible evidence. See
  [`docs/SECURITY_CLAIMS.md`](docs/SECURITY_CLAIMS.md) and
  [`docs/PROOF_OBLIGATIONS.md`](docs/PROOF_OBLIGATIONS.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT License ([`LICENSE-MIT`](LICENSE-MIT))

at your option.
