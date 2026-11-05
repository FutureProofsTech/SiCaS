# Local Proof Runner

`tools/local_proof.sh` is the single local proof entry point. It runs the gates and
writes logs plus a `SUMMARY.md`.

## Quick proof

```bash
./tools/local_proof.sh quick
```

Covers:

- formatting, strict Clippy, and workflow action-pin checks;
- all workspace tests and doctests;
- Kani;
- AddressSanitizer;
- `cargo audit` and `cargo deny check`;
- multi-architecture assembly matrix, including optimization, codegen, panic, and x86_64 LTO variants;
- 20-configuration bitsliced-reference key-plane source and assembly gate across four targets and `opt-level` 0/1/2/3/s;
- Valgrind Memcheck over the workspace library tests;
- zeroization assembly inspection;
- differential statistics validation against an independent Python model;
- first-order leakage-model check with a negative control;
- first-order DPA key recovery on synthetic AES S-box traces;
- release-manifest validator tests;
- source-complete evidence bundle generation with a clean-tracked-tree check.

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

## DPA quick proof

The AES S-box DPA addition was verified by a quick proof on commit `ef13675`:

```bash
LOCAL_PROOF_LOG_DIR=evidence/local-proof-2026-10-31-sbox-dpa ./tools/local_proof.sh quick
```

All quick gates passed, including 141 ASan workspace library tests, the DPA
key-recovery tests, five-binary Valgrind Memcheck, zeroization assembly inspection, and
the evidence-bundle gate. Targeted Miri passes the DPA safe paths (3 passed, 4
native sweeps ignored). AArch64 QEMU and i686 reruns pass 141 and 154 tests
respectively. Coverage on the same tree passes at 97.28% lines and 94.89% regions; the
DPA module is at 100% line coverage. The summary is
[`evidence/local-proof-2026-10-31-sbox-dpa/SUMMARY.md`](../evidence/local-proof-2026-10-31-sbox-dpa/SUMMARY.md).

The full proof predates the DPA addition and the zeroization adapter. Targeted Miri,
workspace mutation, AArch64, i686, and release checks were rerun on `ef13675`; the
full-proof rows remain attributed to `8c41e4b`.

## 2026-11-05 red-team final quick proof

The red-team fixes were verified by a final quick proof on commit `18b5123`:

```bash
LOCAL_PROOF_LOG_DIR=evidence/local-proof-20261105-final ./tools/local_proof.sh quick
```

All quick gates passed: fmt, workflow action-pin checks and their tests, strict
Clippy, workspace tests, doctests, Kani, ASan, audit, deny, the 69-configuration
assembly matrix, zeroization assembly, leakage model, 204-case statistics
differential, Valgrind, release-validator tests, and the source-complete evidence
bundle. Full-workspace Miri also passed on the same code changes. The full proof
(mutation, coverage, five fuzz campaigns, cross-architecture tests, release signing
dry run, and reproducibility comparison) was later rerun at `f3112af`; see the full
proof section below. The summary is
[`evidence/local-proof-20261105-final/SUMMARY.md`](../evidence/local-proof-20261105-final/SUMMARY.md).

## 2026-11-05 CI-fix quick proof

The GitHub mutation job found two misses: an equivalent `1_u64 << 0` mutant in
`BIT_MASKS` and a missing boundary test for `samples == MAX_SAMPLES` in
`leakage::validate`. Both were fixed, the targeted bitsliced-reference key-plane gate
was added, and a quick proof passed on commit `356f70c`:

```bash
LOCAL_PROOF_LOG_DIR=evidence/local-proof-20261105-ci-fix ./tools/local_proof.sh quick
```

The summary is
[`evidence/local-proof-20261105-ci-fix/SUMMARY.md`](../evidence/local-proof-20261105-ci-fix/SUMMARY.md).
Local mutation for `sicas-reference` and `sicas-harness` then reported 530 mutants,
498 caught, 32 unviable, and zero missed.

## 2026-11-05 streaming/reference quick proof

The streaming TVLA and multi-target reference gate were verified by a quick proof on
commit `aaacdea`:

```bash
LOCAL_PROOF_LOG_DIR=evidence/local-proof-20261105-streaming ./tools/local_proof.sh quick
```

All quick gates passed, including the 20-configuration reference key-plane matrix,
streaming TVLA tests, Kani, ASan, Valgrind, and the evidence bundle. The summary is
[`evidence/local-proof-20261105-streaming/SUMMARY.md`](../evidence/local-proof-20261105-streaming/SUMMARY.md).

## 2026-11-05 full local proof

The complete local proof was rerun on commit `f3112af`:

```bash
LOCAL_PROOF_LOG_DIR=evidence/local-proof-20261105-full ./tools/local_proof.sh full
```

All gates passed, including full-workspace Miri, blocking core/reference mutation,
report-only workspace mutation (673 mutants, 617 caught, 0 missed, 56 unviable),
coverage at 95.03% lines and 91.62% regions, five 1M-execution fuzz campaigns,
AArch64 QEMU, i686 native, release manifest/validator, signing dry run, and the
reproducible-build comparison. The bitsliced batch measured 7.515× versus unmasked;
the 10× hard ceiling is met and the 5× target remains open. The summary is
[`evidence/local-proof-20261105-full/SUMMARY.md`](../evidence/local-proof-20261105-full/SUMMARY.md).

## 2026-11-06 full local proof

The full proof was rerun at `b18c7df` after the streaming-TVLA and multi-target
reference-gate work:

```bash
LOCAL_PROOF_LOG_DIR=evidence/local-proof-20261105-final3 ./tools/local_proof.sh full
```

All gates passed, including full-workspace Miri, blocking core/reference mutation,
report-only workspace mutation (714 mutants, 655 caught, 0 missed, 59 unviable),
coverage at 95.06% lines and 91.17% regions, five 1M-execution fuzz campaigns,
AArch64 QEMU, i686 native, release manifest/validator, signing dry run, and the
reproducible-build comparison. The bitsliced batch measured 7.052×; the 10× hard
ceiling is met and the 5× target remains open. The summary is
[`evidence/local-proof-20261105-final3/SUMMARY.md`](../evidence/local-proof-20261105-final3/SUMMARY.md).

## 2026-11-06 final full proof

The final full proof was rerun at `a3495ca` after bounding the streaming loops against
non-terminating readers:

```bash
LOCAL_PROOF_LOG_DIR=evidence/local-proof-20261106-final ./tools/local_proof.sh full
```

All gates passed, including full-workspace Miri, blocking core/reference mutation,
report-only workspace mutation (714 mutants, 655 caught, 0 missed, 0 timeouts, 59
unviable), coverage at 95.05% lines and 91.20% regions, five 1M-execution fuzz
campaigns, AArch64 QEMU, i686 native, release manifest/validator, signing dry run, and
the reproducible-build comparison. The bitsliced batch measured 7.028×; the 10× hard
ceiling is met and the 5× target remains open. The summary is
[`evidence/local-proof-20261106-final/SUMMARY.md`](../evidence/local-proof-20261106-final/SUMMARY.md).

## Interpretation

A PASS in `SUMMARY.md` means the command exited successfully. The workspace mutation
row is report-only; core and reference mutation runs are blocking. The performance gate
is blocking at the 10× hard ceiling and report-only for the 5× target. External-only
gates remain in `docs/GAP_REPORT.md` and are not simulated.

External audit is the second opinion after this local proof package is complete.
