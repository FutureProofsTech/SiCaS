# Auditor Guide

This is the shortest path for an independent reviewer to understand, reproduce, and
attack SiCaS. It is a review aid, not evidence that an independent review has happened.

## Reviewed state

- Code commit: `ef13675`
- Release build commit: `123eb69`
- Evidence commit/tag: `final-audit-2026-10-31-dpa`
- Public label: **SAL-2 locally**
- Release SHA-256: `1f0d304ca9b9582fd9153e7c006c240bd0e06acc3a685dfad93088429e4de7da`
- Release binary size: 386,232 bytes
- Masked ChaCha20: 7.15× vs unmasked in the final full proof, 10× hard ceiling met

## What to read first

1. `docs/THREAT_MODEL.md` — assets, adversary, non-goals.
2. `docs/SECURITY_CLAIMS.md` — exact claims and non-claims.
3. `docs/BITSLICED_MASKING.md` — the masking construction and its limits.
4. `docs/PROOF_OBLIGATIONS.md` — property-to-evidence map.
5. `docs/TEST_MATRIX.md` — which test proves which property.
6. `docs/GAP_REPORT.md` — what is not proven locally.

## Five-minute smoke test

```bash
cargo test -p sicas-core -p sicas-reference --all-targets --all-features --locked
cargo run --release -p sicas-harness -- leakage-check --samples 50000
python3 tools/ct_asm_check.py --matrix --lto --output /tmp/sicas-asm.md
tools/run-valgrind.sh /tmp/sicas-valgrind
```

Expected: all tests pass, the negative control reports correlation 1.0, every masked
check stays below 0.02, the assembly matrix reports no conditional branches, and
Valgrind reports zero errors across five test binaries.

## One-command local proof

```bash
./tools/local_proof.sh quick   # fast gate set
./tools/local_proof.sh full    # full proof, roughly 30–45 minutes
```

The full run writes `SUMMARY.md` and raw logs under `evidence/local-proof-<timestamp>/`.
It covers fmt, strict Clippy, tests, doctests, Miri, Kani, ASan, Valgrind, mutation,
coverage, five fuzz campaigns including the 64-block bitsliced batch, cross-architecture
tests, leakage model, AES S-box DPA with synthetic key recovery, statistics differential, zeroization-boundary tests, release
validation, signing dry run, and reproducible builds.

## Verify the evidence bundle

```bash
cd evidence/bundle
sha256sum -c MANIFEST.sha256
```

The bundle index records the code commit and every file hash. The local proof summary is
also bundled.

## Claim-to-evidence map

| Claim | Primary evidence | Command |
|---|---|---|
| C1 safe secret API | compile-fail doctests, `Secret<T>` API | `cargo test --workspace --doc` |
| C2 constant time | assembly matrix, dudect reports | `python3 tools/ct_asm_check.py --matrix --lto`; `sicas-harness ct` |
| C3 first-order masking | Kani gadgets, bitsliced lane test, leakage model | `cargo test -p sicas-reference`; `leakage-check` |
| C4 entropy fail-closed | entropy failure tests | `cargo test -p sicas-core -p sicas-platform` |
| C5 zero runtime deps | `cargo tree`, `cargo metadata` | `cargo tree --workspace --edges normal` |
| C6 no unsafe in published crates | crate attributes, source audit | `grep -R "forbid(unsafe_code)" crates` |
| C7 reproducible builds | three-build hash comparison | full proof |
| C8 model-checked invariants | 30 Kani harnesses | `cargo kani --workspace` |
| C9 SAL-4 | not claimed | see `docs/GAP_REPORT.md` |
| C10 signed release | local dry run only | see `docs/RELEASE.md` |

## Where to attack

- **Masking composition:** `crates/sicas-reference/src/chacha20.rs`, especially
  `keystream_blocks`, `add_planes`, `transpose_word`, and `quarter_round_bitsliced`.
- **Core gadgets:** `crates/sicas-core/src/masked.rs` and `ct.rs`.
- **Entropy failure paths:** `crates/sicas-core/src/entropy.rs` and
  `crates/sicas-platform/src/entropy.rs`.
- **Parser/DoS:** `crates/sicas-harness/src/trace.rs`, fuzz target `trace_parser`.
- **Statistics:** `crates/sicas-harness/src/{tvla,cpa,stats}.rs` and
  `tools/check_stats.py`.
- **Release integrity:** `tools/release_manifest.py`,
  `tools/validate_release_manifest.py`, and `evidence/bundle`. The release manifest
  records build commit `123eb69` (proof code commit `ef13675`); the evidence tag points
  to a later docs/evidence commit, so add `--skip-commit` when validating from the tag.

## Known limits

- No physical power/EM TVLA/CPA/DPA.
- No ARM/RISC-V runtime timing on hardware.
- No independent audit or red team yet.
- No production signing identity or SLSA attestation.
- Zeroization has an isolated, experimental, unaudited adapter (`sicas-zeroize`).
- The 5× performance target is open; the 10× hard ceiling is met.

## Reporting

Follow `SECURITY.md`. Include the commit, a minimal reproduction, the affected claim,
and whether the issue is inside the declared threat model. Use synthetic keys and
isolated systems only.
