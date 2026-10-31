# Masked ChaCha20 Performance Evidence

Date: 2026-10-31

Commit: `6ad9c10` (full local-proof run)

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
scalar masked: 312.962x vs unmasked
batch masked:  7.264x vs unmasked
batch ns/byte: 18.835434
unmasked ns/byte: 2.592857
target_ratio<=5 hard_ceiling<=10
sicas-harness release size: 383,160 bytes (was 625,080)
```

## Interpretation

The scalar reference is approximately **331× slower** than the unmasked reference in the full-proof run. The
bitsliced batch path is **7.26×** slower, so the hard ceiling of 10× is **met** with
margin. The initial ≤5× target is **not met**.

The release profile uses `opt-level=z`, fat LTO, one codegen unit, panic=abort, and
stripped symbols. That is both faster and smaller than the previous `opt-level=3`
profile on this workload. The scalar `Masked::add_masked` latency regresses under this
profile (about 550 ns vs 134 ns in dudect), while the bitsliced batch throughput
improves; this is a deliberate throughput/size tradeoff, not a security claim. The remaining gap is in the bitsliced masked adder and
entropy stream; wider SIMD lanes or a faster audited mask stream would be the next
step.

The hard ceiling is met. The remaining 5× target is recorded as an open local
optimization target, not hidden by weakening the benchmark.

## Limitations

- Single host, single run; use multiple runs and a quiet machine for a stable number.
- The unmasked reference is a simple scalar implementation, not an optimized ChaCha20.
- The benchmark includes deterministic entropy generation; a production CSPRNG would
  add cost.
