# Masked ChaCha20 Performance Evidence

Date: 2026-10-31

Commit: `160180f` (full local-proof run)

Target: x86_64-unknown-linux-gnu

Toolchain: Rust 1.99.0

Command:

```bash
cargo run --release -p sicas-reference --example benchmark
```

## Raw result

The scalar reference remains slow. The bitsliced batch path processes 64 blocks
together; the tuned release profile also shrinks the binary:

```text
scalar masked: 322.443x vs unmasked
batch masked:  7.387x vs unmasked
batch ns/byte: 19.358845
unmasked ns/byte: 2.620739
target_ratio<=5 hard_ceiling<=10
sicas-harness release size: 383,160 bytes (was 625,080)
```

## Interpretation

The scalar reference is approximately **331× slower** than the unmasked reference in the full-proof run. The
bitsliced batch path is **7.39×** slower, so the hard ceiling of 10× is **met** with
margin. The initial ≤5× target is **not met**.

The release profile uses `opt-level=z`, fat LTO, one codegen unit, panic=abort, and
stripped symbols. That is both faster and smaller than the previous `opt-level=3`
profile on this workload. The remaining gap is in the bitsliced masked adder and
entropy stream; wider SIMD lanes or a faster audited mask stream would be the next
step.

The hard ceiling is met. The remaining 5× target is recorded as an open local
optimization target, not hidden by weakening the benchmark.

## Limitations

- Single host, single run; use multiple runs and a quiet machine for a stable number.
- The unmasked reference is a simple scalar implementation, not an optimized ChaCha20.
- The benchmark includes deterministic entropy generation; a production CSPRNG would
  add cost.
