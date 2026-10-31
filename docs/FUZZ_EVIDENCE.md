# Fuzz Evidence

## Campaign

Date: 2026-10-30  
Commit: `53be7e2` (ChaCha20 campaign after audit fix `45a8cce`)  
Target: x86_64-unknown-linux-gnu  
Tool: `cargo-fuzz` 0.13.2 / libFuzzer

Each target was run for 1,000,000 executions:

```bash
cargo +nightly-2026-08-03 fuzz run masked_ops -- -runs=1000000
cargo +nightly-2026-08-03 fuzz run ct_lookup -- -runs=1000000
cargo +nightly-2026-08-03 fuzz run trace_parser -- -runs=1000000
cargo +nightly-2026-08-03 fuzz run chacha20 -- -runs=1000000
cargo +nightly-2026-08-03 fuzz run chacha20_batch -- -runs=1000000
```

| Target | Executions | Result | Notes |
|---|---:|---|---|
| `masked_ops` | 1,000,000 | no crashes | masked arithmetic invariants |
| `ct_lookup` | 1,000,000 | no crashes | 256-byte constant-time lookup |
| `trace_parser` | 1,000,000 | no crashes | malformed trace input |
| `chacha20` | 1,000,000 | no crashes | masked output vs reference |
| `chacha20_batch` | 1,000,000 | no crashes | 64-block bitsliced batch vs reference |

The `chacha20_batch` target processes 64 blocks per input, so its execution rate is
lower than the scalar targets; it still receives a full 1,000,000-execution campaign.

## Long campaign — 2026-10-31

```bash
tools/run-fuzz-long.sh evidence/fuzz-long
```

| Target | Executions | Result |
|---|---:|---|
| `masked_ops` | 10,000,000 | no crashes |
| `ct_lookup` | 10,000,000 | no crashes |
| `trace_parser` | 10,000,000 | no crashes |
| `chacha20` | 1,000,000 | no crashes |

The masked ChaCha20 target is substantially slower, so the long campaign keeps its
1,000,000-execution run while the three fast targets run 10,000,000 executions. The
summary and per-target logs are in
[`evidence/fuzz-long/SUMMARY.md`](../evidence/fuzz-long/SUMMARY.md).

## Limitations

- One local x86_64 campaign is not proof of correctness.
- Fuzzing cannot prove the absence of bugs.
- Continuous fuzzing and corpus management remain required.
- Hardware-specific behavior is not covered by these targets.
