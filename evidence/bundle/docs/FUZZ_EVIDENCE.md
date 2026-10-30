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
```

| Target | Executions | Result | Notes |
|---|---:|---|---|
| `masked_ops` | 1,000,000 | no crashes | masked arithmetic invariants |
| `ct_lookup` | 1,000,000 | no crashes | 256-byte constant-time lookup |
| `trace_parser` | 1,000,000 | no crashes | malformed trace input |
| `chacha20` | 1,000,000 | no crashes | masked output vs reference |

## Limitations

- One local x86_64 campaign is not proof of correctness.
- Fuzzing cannot prove the absence of bugs.
- Continuous fuzzing and corpus management remain required.
- Hardware-specific behavior is not covered by these targets.
