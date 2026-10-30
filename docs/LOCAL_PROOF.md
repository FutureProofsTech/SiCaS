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
- multi-architecture assembly checks;
- first-order leakage-model check with a negative control;
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
- four 1M-execution fuzz campaigns;
- AArch64 QEMU functional tests;
- i686 native all-target tests;
- release manifest, SBOM, provenance, validator, and reproducible-build comparison;
- final evidence bundle.

The full run can take 20+ minutes. It writes everything under
`evidence/local-proof-<timestamp>/` unless `LOCAL_PROOF_LOG_DIR` is set.

## Final local proof

The frozen local proof ran on commit `c349329`:

```bash
LOCAL_PROOF_LOG_DIR=evidence/local-proof-2026-10-31 ./tools/local_proof.sh full
```

All gates passed, including full-workspace Miri, 27 Kani harnesses, 122 ASan tests,
zero missed viable mutants, 96.90% line coverage, four 1M-execution fuzz campaigns,
AArch64 QEMU, i686, release validation, and the reproducible-build comparison. The
summary is [`evidence/local-proof-2026-10-31/SUMMARY.md`](../evidence/local-proof-2026-10-31/SUMMARY.md).

## Interpretation

A PASS in `SUMMARY.md` means the command exited successfully. A report-only mutation
line records accepted misses in the analysis harness; core and reference mutation runs
are blocking. External-only gates remain in `docs/GAP_REPORT.md` and are not simulated.

External audit is the second opinion after this local proof package is complete.
