# SiCaS

Safe, zero-dependency Rust primitives for side-channel attenuation and leakage
assessment.

## Status

Early Phase 1. The core crate is not audited, not hardware-validated, and **must not
be used to protect real secrets yet**. The production target is SAL-4 as defined in
[`PLAN.md`](PLAN.md) and [`docs/ASSURANCE.md`](docs/ASSURANCE.md).

## What exists

- `sicas-core`: `no_std`, `#![forbid(unsafe_code)]`, zero runtime dependencies.
  - `Choice` and constant-time comparison/selection/swap/lookup.
  - `Secret<T>` with redacted `Debug` and no accidental equality/indexing/cloning.
  - `Masked<u32>` with two-share first-order Boolean masking.
  - `EntropySource` with fail-closed behavior.
- Unit tests, exhaustive byte tests, property-style randomized tests, and compile-fail
  doctests.
- Strict formatting and Clippy policy.

## Quality gates

```bash
cargo +nightly-2026-08-03 fmt --all -- --check

cargo +nightly-2026-08-03 clippy \
  --workspace --all-targets --all-features --locked \
  -- -D warnings -D clippy::pedantic -D clippy::nursery

cargo test --workspace --all-targets --all-features --locked
cargo test --workspace --doc --locked
cargo +nightly-2026-08-03 miri test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --locked
```

## Constraints

- Safe Rust: `sicas-core` forbids unsafe code.
- Zero runtime dependencies: no third-party crates.
- Every security claim must link to reproducible evidence. See
  [`docs/SECURITY_CLAIMS.md`](docs/SECURITY_CLAIMS.md) and
  [`docs/PROOF_OBLIGATIONS.md`](docs/PROOF_OBLIGATIONS.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT License ([`LICENSE-MIT`](LICENSE-MIT))

at your option.
