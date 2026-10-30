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
  - Fixed-vs-random TVLA with Welch's t-test.
  - Byte-oriented Hamming-weight CPA.
  - Deterministic synthetic trace generation.
  - Dudect-style timing tooling.
  - CLI: `tvla`, `cpa`, `synthetic`, `synthetic-cpa`, `timing`.
- Unit tests, exhaustive byte tests, property-style randomized tests, compile-fail
  doctests, Kani proof harnesses, and libFuzzer targets.
- AddressSanitizer, Miri, strict formatting, and strict Clippy gates.

See [`docs/EVALUATION.md`](docs/EVALUATION.md) for trace format and usage.

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
