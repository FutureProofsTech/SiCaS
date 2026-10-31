# Constant-Time Evidence

## Method

The `sicas-harness ct` command runs a dudect-style fixed-vs-random timing test on:

- `ct::eq_u32`
- `ct::select_u32`
- `ct::lookup_u8`
- `Masked::<u32>::add_masked`

Each trial uses 1,000,000 measured samples and 100,000 warm-up calls. Five trials are
run per check. The leakage threshold is `|t| > 4.5`. Inputs, entropy bytes, masks, and
the lookup table are passed through `black_box` to prevent constant folding.

```bash
cargo run -p sicas-harness -- ct \
  --samples 1000000 --warmup 100000 --trials 5 \
  --label "db8f77c x86_64-unknown-linux-gnu rustc 1.99.0" \
  --report evidence/ct/2026-10-30-x86_64.md
```

## Results — 2026-10-30, x86_64

| Check | Max absolute t across 5 trials | Result |
|-------|--------------------------------|--------|
| `ct::eq_u32` | 1.0010 | pass |
| `ct::select_u32` | 1.1861 | pass |
| `ct::lookup_u8` | 0.9013 | pass |
| `Masked::<u32>::add_masked` | 2.2472 | pass |

All trials are below the `|t| > 4.5` threshold. The complete per-trial report is in
[`evidence/ct/2026-10-30-x86_64.md`](../evidence/ct/2026-10-30-x86_64.md).

## Release-profile timing — 2026-10-31

The release-profile run uses the tuned `opt-level=z` profile and the same five-check suite:

```bash
tools/run-release-timing.sh evidence/ct/2026-10-31-x86_64-release.md
```

| Check | Max absolute t across 3 trials | Result |
|-------|--------------------------------|--------|
| `ct::eq_u32` | 1.0297 | pass |
| `ct::select_u32` | 1.5631 | pass |
| `ct::lookup_u8` | 0.8523 | pass |
| `Masked::<u32>::add_masked` | 0.6469 | pass |

All trials are below `|t| > 4.5`. The full report is in
[`evidence/ct/2026-10-31-x86_64-release.md`](../evidence/ct/2026-10-31-x86_64-release.md).

## Assembly inspection

Run the assembly matrix checker:

```bash
python3 tools/ct_asm_check.py --matrix --lto \
  --output evidence/ct/asm-2026-10-31.md
```

The default rows build `sicas-core` at `-C opt-level=0` for x86_64, AArch64,
Thumbv7em, and RISC-V, extract `ct::eq_u32`, `ct::select_u32`, and
`ct::is_zero_u32`, and reject conditional branches.

The matrix rows build the `ct_asm` example on x86_64 and AArch64 for:

- debug `opt-level=0` and `1`;
- release `opt-level=2`, `3`, `s`, and `z`;
- release `codegen-units=1`;
- release `panic=abort`.

The LTO rows build linked x86_64 binaries with thin and fat LTO, including the
`opt-level=z` profile, and inspect the disassembly.

All 60 inspected configurations report no conditional branches in the three
constant-time functions. The full generated report is in
[`evidence/ct/asm-2026-10-31.md`](../evidence/ct/asm-2026-10-31.md).
`ct::lookup_u8` is excluded from this check because it intentionally contains the
public 256-iteration loop branch; its selection is performed by constant-time
equality.

This is supporting evidence for the inspected targets and configurations. It is not a
proof for all architectures, all LTO combinations, or microarchitectural states.

## Limitations

- Timing evidence is x86_64 Linux only in this report.
- Timing is environment-sensitive; the report records the exact sample count, trials,
  threshold, toolchain, and commit.
- Assembly inspection covers opt-level 0 for four targets, an optimization/codegen/
  panic matrix on x86_64 and AArch64, and thin/fat LTO on x86_64. Other architectures
  and LTO combinations still need inspection.
- No hardware EM/power evidence yet.
- A passing timing test does not prove absence of all microarchitectural leakage.
