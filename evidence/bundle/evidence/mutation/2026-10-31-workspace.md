# Workspace Mutation Evidence

- Date: 2026-10-31
- Commit: `ef13675a9d44e881e1420f4cd5b9313043f13b73`
- Tool: `cargo-mutants 27.1.0`
- Command: `cargo mutants --workspace --no-shuffle`
- Targeted command: `cargo mutants --file crates/sicas-harness/src/dpa.rs --no-shuffle`

## Results

| Scope | Mutants | Caught | Missed | Unviable |
|---|---:|---:|---:|---:|
| Workspace | 630 | 574 | 0 | 56 |
| `crates/sicas-harness/src/dpa.rs` | 30 | 29 | 0 | 1 |

The workspace run covers every crate, including the AES S-box DPA, the zeroization
drop path, and the masked ChaCha20 reference. The targeted DPA run covers every mutant
generated in `dpa.rs`; the single unviable mutant is rejected by the compiler and is not
a test gap.

## Full-proof comparison

The full proof at `8c41e4b` recorded 600 workspace mutants: 544 caught, 1 missed, and 55
unviable. The one miss was the `ZeroizingBytes` drop path, which had no test at that
commit. Commit `92af463` added the drop-path test, and the current run catches it.

## Raw logs

- `evidence/mutation/2026-10-31-workspace.log`
- `evidence/mutation/2026-10-31-dpa.log`

Mutation testing measures the strength of the local test suite, not the absence of
defects. It does not replace Miri, Kani, fuzzing, hardware leakage measurement, or
independent review.
