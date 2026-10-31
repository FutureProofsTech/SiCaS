# Mutation Testing Evidence

- Date: 2026-10-31
- Commit: `ac06c1c` (full local-proof commit)
- Tool: `cargo-mutants 27.1.0`
- Command: `cargo mutants --workspace --no-shuffle`
- Per-crate commands: `cargo mutants -p <crate> --no-shuffle`

## Results

| Scope | Caught | Missed | Unviable | Viable mutants |
|---|---:|---:|---:|---:|
| `sicas-core` | 112 | 0 | 19 | 112 |
| `sicas-harness` | 357 | 0 | 23 | 357 |
| `sicas-reference` | 69 | 0 | 8 | 69 |
| `sicas-platform` | 2 | 0 | 1 | 2 |
| **Workspace** | **542** | **0** | **51** | **542** |

All viable mutants were caught in the final full proof. The reference crate accounts
for 69 caught, 0 missed, and 8 unviable mutants. Unviable mutants are those that
cargo-mutants could not build or that the compiler rejected; they are excluded from the
score.

## What this covers

- Core constant-time gadgets, masking, refresh, share reconstruction, addition, and
  parser limits.
- Harness trace parsing, preprocessing, TVLA, CPA, timing validation, synthetic
  generation, and leakage-model checks.
- Reference masked ChaCha20 and entropy-failure paths.
- Platform entropy adapter error paths.

## Limitations

Mutation testing measures the strength of the local test suite, not the absence of
defects. It does not replace Miri, Kani, fuzzing, hardware leakage measurement, or
independent review. The harness CLI binary is excluded from the mutation score; its
branchless timing-input selection is behaviorally equivalent for the measured property
and is covered by CLI integration tests.
