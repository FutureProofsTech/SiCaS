# Local Proof Runner

`tools/local_proof.sh` is the single local proof entry point. It runs the gates and
writes logs plus a `SUMMARY.md`.

## Quick proof

```bash
./tools/local_proof.sh quick
```

Covers:

- formatting and strict Clippy;
- all workspace tests and doctests;
- Kani;
- AddressSanitizer;
- `cargo audit` and `cargo deny check`;
- multi-architecture assembly matrix, including optimization, codegen, panic, and x86_64 LTO variants;
- Valgrind Memcheck over the workspace library tests;
- zeroization assembly inspection;
- differential statistics validation against an independent Python model;
- first-order leakage-model check with a negative control;
- first-order DPA key recovery on synthetic AES S-box traces;
- release-manifest validator tests;
- evidence bundle generation.

## Full proof

```bash
./tools/local_proof.sh full
```

Adds:

- full workspace Miri;
- core and reference mutation testing;
- report-only workspace mutation testing;
- coverage with a 90% line threshold;
- five 1M-execution fuzz campaigns;
- AArch64 QEMU functional tests;
- i686 native all-target tests;
- release manifest, SBOM, provenance, validator, local signing dry run, and reproducible-build comparison;
- final evidence bundle.

The full run can take 30–45 minutes. It writes everything under
`evidence/local-proof-<timestamp>/` unless `LOCAL_PROOF_LOG_DIR` is set.

## Full local proof

The full proof ran on commit `8c41e4b`:

```bash
LOCAL_PROOF_LOG_DIR=evidence/local-proof-2026-10-31-final-audit ./tools/local_proof.sh full
```

All gates passed, including full-workspace Miri, 30 Kani harnesses, 133 ASan tests,
zero missed viable mutants in the blocking core and reference runs, 97.23% line
coverage, five 1M-execution fuzz campaigns, AArch64 QEMU, i686, release validation,
the local signing dry run, and the reproducible-build comparison. The performance gate
passed at 7.15×, meeting the 10× hard ceiling; the 5× target remains open.

The report-only workspace mutation run recorded one miss in the then-untested
`ZeroizingBytes` drop path. Commit `92af463` added the drop-path test, and the
workspace mutation rerun records zero missed viable mutants. The summary is
[`evidence/local-proof-2026-10-31-final-audit/SUMMARY.md`](../evidence/local-proof-2026-10-31-final-audit/SUMMARY.md).

## Interpretation

A PASS in `SUMMARY.md` means the command exited successfully. The workspace mutation
row is report-only; core and reference mutation runs are blocking. The performance gate
is blocking at the 10× hard ceiling and report-only for the 5× target. External-only
gates remain in `docs/GAP_REPORT.md` and are not simulated.

External audit is the second opinion after this local proof package is complete.
