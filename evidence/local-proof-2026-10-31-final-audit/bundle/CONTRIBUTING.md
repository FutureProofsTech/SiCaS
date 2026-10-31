# Contributing

## Before submitting

Run the full local gate:

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

## Rules

- No third-party runtime dependencies.
- No `unsafe` in `sicas-core`.
- No secret-dependent branches or indexing.
- No `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, or `dbg!` in production paths.
- Every public item needs documentation and tests.
- Every security claim needs a proof obligation and evidence link.
- Security-critical changes need an independent review before merge.

## Commit style

Use short, imperative commit subjects. Keep security-relevant changes isolated from
formatting-only changes.
