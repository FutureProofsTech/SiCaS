# Mutation Testing Evidence

- Date: 2026-10-31
- Tool: `cargo-mutants 27.1.0`
- Command: `cargo mutants --workspace --no-shuffle`
- Per-crate commands: `cargo mutants -p <crate> --no-shuffle`
- Targeted file command: `cargo mutants --file crates/sicas-harness/src/dpa.rs --no-shuffle`

## Full-proof results at `8c41e4b`

| Scope | Caught | Missed | Unviable | Viable mutants |
|---|---:|---:|---:|---:|
| `sicas-core` (blocking) | 114 | 0 | 19 | 114 |
| `sicas-reference` (blocking) | 69 | 0 | 8 | 69 |
| Workspace (report-only) | 544 | 1 | 55 | 545 |

The blocking core and reference runs caught every viable mutant. The report-only
workspace run missed one mutant in the `ZeroizingBytes` drop path, which had no test at
that commit. Commit `92af463` added the drop-path test, and the later workspace runs
catch it. Unviable mutants are those that cargo-mutants could not build or that the
compiler rejected; they are excluded from the score.

## Current-tree results at `ef13675`

| Scope | Caught | Missed | Unviable | Viable mutants |
|---|---:|---:|---:|---:|
| Workspace | 574 | 0 | 56 | 574 |
| `crates/sicas-harness/src/dpa.rs` (targeted) | 29 | 0 | 1 | 29 |

The workspace run covers the AES S-box DPA, the zeroization drop path, and the rest of
the workspace. The targeted DPA run covers all 30 mutants generated in `dpa.rs`; the one
unviable mutant is not a test gap. Raw logs are in
`evidence/mutation/2026-10-31-workspace.log` and
`evidence/mutation/2026-10-31-dpa.log`.

## What this covers

- Core constant-time gadgets, masking, refresh, share reconstruction, addition, and
  parser limits.
- Harness trace parsing, preprocessing, TVLA, CPA, DPA, timing validation, synthetic
  generation, and leakage-model checks.
- Reference masked ChaCha20 and entropy-failure paths.
- Platform entropy adapter error paths.
- The isolated `sicas-zeroize` volatile-write and drop paths.

## Limitations

Mutation testing measures the strength of the local test suite, not the absence of
defects. It does not replace Miri, Kani, fuzzing, hardware leakage measurement, or
independent review. The harness CLI binary is excluded from the mutation score; its
branchless timing-input selection is behaviorally equivalent for the measured property
and is covered by CLI integration tests.
