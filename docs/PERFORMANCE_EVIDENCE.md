# Masked ChaCha20 Performance Evidence

Date: 2026-10-31

Commit: bitsliced batch implementation (see repository history)

Target: x86_64-unknown-linux-gnu

Toolchain: Rust 1.99.0

Command:

```bash
cargo run --release -p sicas-reference --example benchmark
```

## Raw result

The scalar reference remains slow. The bitsliced batch path processes 64 blocks
together and meets the hard ceiling:

```text
scalar masked: ~600x vs unmasked
batch masked:  9.31x to 9.59x vs unmasked
batch ns/byte: 12.19 to 12.53
unmasked ns/byte: 1.31
target_ratio<=5 hard_ceiling<=10
```

## Interpretation

The scalar reference is approximately **600× slower** than the unmasked reference. The
bitsliced batch path is **9.3×–9.6×** slower, so the hard ceiling of 10× is **met**. The
initial ≤5× target is **not met**.

The remaining gap is in the bitsliced masked adder and entropy stream. A production
implementation would need wider SIMD lanes, a faster audited mask stream, or a
different masked-adder representation. The current batch path is safe Rust, processes
64 blocks per mask stream, and uses a fast 64×64 bit-matrix transpose for output.

This result is recorded as an open local engineering gap. It is not hidden by
weakening the benchmark, and no performance claim is made.

## Limitations

- Single host, single run; use multiple runs and a quiet machine for a stable number.
- The unmasked reference is a simple scalar implementation, not an optimized ChaCha20.
- The benchmark includes deterministic entropy generation; a production CSPRNG would
  add cost.
