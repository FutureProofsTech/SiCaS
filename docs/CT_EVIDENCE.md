# Constant-Time Evidence

## Method

The `sicas-harness ct` command runs a dudect-style fixed-vs-random timing test on:

- `ct::eq_u32`
- `ct::select_u32`
- `ct::lookup_u8`
- `Masked::<u32>::add_masked`

Each trial uses 1,000,000 measured samples and 100,000 warm-up calls. Five trials are
run per check. The leakage threshold is `|t| > 4.5`.

```bash
cargo run -p sicas-harness -- ct \
  --samples 1000000 --warmup 100000 --trials 5 \
  --label "c44a5dd x86_64-unknown-linux-gnu rustc 1.99.0" \
  --report evidence/ct/2026-10-30-x86_64.md
```

## Results — 2026-10-30, x86_64

| Check | Max absolute t across 5 trials | Result |
|-------|--------------------------------|--------|
| `ct::eq_u32` | 1.6663 | pass |
| `ct::select_u32` | 1.5604 | pass |
| `ct::lookup_u8` | 0.7245 | pass |
| `Masked::<u32>::add_masked` | 0.9782 | pass |

All trials are below the `|t| > 4.5` threshold. The complete per-trial report is in
[`evidence/ct/2026-10-30-x86_64.md`](../evidence/ct/2026-10-30-x86_64.md).

## Assembly inspection

Build the inspection target:

```bash
cargo rustc -p sicas-core --example ct_asm -- --emit=asm -C opt-level=0
```

Inspect the generated assembly in `target/debug/examples/ct_asm-*.s` and the core
library assembly in `target/debug/deps/sicas_core-*.s`.

For `ct::eq_u32`, the x86_64 body is a single XOR followed by a call to
`ct::is_zero_u32`; there is no conditional branch on the operands. For
`ct::select_u32`, the body is mask construction followed by XOR/AND/XOR; there is no
conditional branch on the choice or operands. `ct::lookup_u8` contains only the
public loop-control branch over all 256 candidates; the selected entry is chosen by
constant-time equality, not by a secret-dependent index branch.

This is supporting evidence for the inspected target and toolchain. It is not a proof
for all architectures or optimization levels.

## Limitations

- x86_64 Linux only in this report.
- Timing is environment-sensitive; the report records the exact sample count, trials,
  threshold, toolchain, and commit.
- Assembly inspection was performed at `-C opt-level=0`; release inlining and
  architecture-specific code generation must be re-inspected for every supported
  target.
- No ARM, RISC-V, or hardware EM/power evidence yet.
- A passing timing test does not prove absence of all microarchitectural leakage.
