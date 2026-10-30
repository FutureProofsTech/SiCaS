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

Run the multi-target assembly checker:

```bash
python3 tools/ct_asm_check.py --output evidence/ct/asm-2026-10-30.md
```

It builds `sicas-core` at `-C opt-level=0` for x86_64, AArch64, Thumbv7em, and
RISC-V, extracts `ct::eq_u32`, `ct::select_u32`, and `ct::is_zero_u32`, and rejects
conditional branches.

| Target | `eq_u32` | `select_u32` | `is_zero_u32` |
|---|---|---|---|
| x86_64 | no branches | no branches | no branches |
| AArch64 | no branches | no branches | no branches |
| Thumbv7em | no branches | no branches | no branches |
| RISC-V 32 | no branches | no branches | no branches |

The full generated report is in
[`evidence/ct/asm-2026-10-30.md`](../evidence/ct/asm-2026-10-30.md).
`ct::lookup_u8` is excluded from this check because it intentionally contains the
public 256-iteration loop branch; its selection is performed by constant-time
equality.

This is supporting evidence for the inspected targets and optimization level. It is
not a proof for all architectures, optimization levels, or microarchitectural states.

## Limitations

- Timing evidence is x86_64 Linux only in this report.
- Timing is environment-sensitive; the report records the exact sample count, trials,
  threshold, toolchain, and commit.
- Assembly inspection was performed at `-C opt-level=0`; release inlining and
  architecture-specific code generation must be re-inspected for every supported
  target and optimization level.
- No hardware EM/power evidence yet.
- A passing timing test does not prove absence of all microarchitectural leakage.
