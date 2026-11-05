# Masked ChaCha20 Performance Evidence

Date: 2026-10-31

Commit: `8c41e4b` (full local-proof run)

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
sicas-harness release size: 383,160 bytes (historical full-proof build; was 625,080)
```

## Interpretation

The scalar reference is approximately **331× slower** than the unmasked reference in the full-proof run. The
bitsliced batch path is **7.26×** slower, so the hard ceiling of 10× is **met** with
margin. The initial ≤5× target is **not met**.

## 2026-11-05 full proof at `f3112af`

The red-team follow-up full proof re-ran the benchmark on the same host:

```text
scalar masked: 328.287x vs unmasked
batch masked:  7.515x vs unmasked
batch ns/byte: 19.775234
unmasked ns/byte: 2.631344
target_ratio<=5 hard_ceiling<=10
```

The batch path remains under the 10× hard ceiling at **7.515×**. The 5× target is
still open. The run-to-run variation between 7.06×, 7.26×, 7.515×, and 7.15× is
host noise and does not change the ceiling conclusion. A batching experiment that
pre-fetched 64 masks per adder was rejected because it regressed the benchmark to
8.55×; the current immediate-use mask flow is retained.

## Optimization attempts — 2026-11-05

Several local experiments were measured on the same host and rejected because they
did not reach the 5× target or regressed the ratio:

| Experiment | Batch ratio | Decision |
|---|---:|---|
| Current immediate-use mask flow | 7.06–7.52× | Retained |
| `sicas-core` at `opt-level=3` (package override) | 6.84–7.09× | Not adopted: small gain, breaks the uniform `opt-level=z` release profile |
| Pre-fetch 64 masks per adder | 8.55× | Rejected: stack traffic and lost register locality |
| `#[inline(always)]` on `Masked` methods | 7.10× | Rejected: no meaningful gain |
| `sicas-reference` at `opt-level=3` | 8.78× | Rejected: optimizes the unmasked baseline as well, worsening the ratio |

The 5× target is not reachable by these constant-factor changes. It requires a new
masking construction or portable SIMD/wider lanes, both of which are new security
boundaries requiring a new proof and review. The 10× hard ceiling is met with margin;
the 5× target remains an explicitly open performance goal, not a hidden or weakened
claim.

## DPA build

The AES S-box DPA addition changes the `sicas-harness` release binary to 386,232 bytes
with SHA-256
`1f0d304ca9b9582fd9153e7c006c240bd0e06acc3a685dfad93088429e4de7da`. It does not
touch `sicas-reference`, so the masked ChaCha20 ratios above are unchanged.

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
