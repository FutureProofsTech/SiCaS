# Masked ChaCha20 Performance Evidence

Date: 2026-10-31

Commit: working tree after `c349329`

Target: x86_64-unknown-linux-gnu

Toolchain: Rust 1.99.0

Command:

```bash
cargo run --release -p sicas-reference --example benchmark
```

## Raw result

```text
masked_blocks=4096
masked_ns_per_byte=780.155254
unmasked_ns_per_byte=1.306652
ratio=597.064
target_ratio<=5 hard_ceiling<=10
```

## Interpretation

The masked reference is approximately **597× slower** than the unmasked reference on
this host. The PLAN's initial budget is ≤5× with a hard ceiling of 10×. The budget is
**not met**.

The gap is expected for the current bit-level ripple-carry masked adder: every
ChaCha20 addition performs 32 bit steps with three masked AND gates and fresh masks.
This reference exists to validate masking composition, not to meet production
throughput. A production implementation would need a bitsliced or word-level masked
adder, fewer masks per addition, or a different masking representation.

This result is recorded as an open local engineering gap. It is not hidden by
weakening the benchmark, and no performance claim is made.

## Limitations

- Single host, single run; use multiple runs and a quiet machine for a stable number.
- The unmasked reference is a simple scalar implementation, not an optimized ChaCha20.
- The benchmark includes deterministic entropy generation; a production CSPRNG would
  add cost.
