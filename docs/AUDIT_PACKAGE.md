# Audit Package

This package describes what an independent auditor or red team should review. It is a
request for a **second opinion**, not evidence that a review has occurred.

Nothing in this package is submitted externally until the local proof package is
complete: every locally provable claim has evidence, every local gate is green, and
every remaining gap is explicitly hardware-, credential-, or independent-party-only.

## Scope

- `sicas-core`: constant-time primitives, two-share Boolean masking, masked addition.
- `sicas-reference`: masked ChaCha20 reference integration.
- `sicas-harness`: trace format, preprocessing, TVLA, CPA, timing, CLI.
- `sicas-platform`: OS entropy adapter.
- Build, CI, release, SBOM, and reproducibility paths.
- Threat model, claims matrix, and proof obligations.

## Evidence index

| Evidence | Location |
|---|---|
| Threat model | `docs/THREAT_MODEL.md` |
| Claims and non-claims | `docs/SECURITY_CLAIMS.md` |
| Proof obligations | `docs/PROOF_OBLIGATIONS.md` |
| Assurance program | `docs/ASSURANCE.md` |
| Code-quality standard | `docs/CODE_QUALITY.md` |
| Constant-time timing | `evidence/ct/2026-10-30-x86_64.md` |
| Assembly checks | `evidence/ct/asm-2026-10-30.md` |
| First-order leakage model | `evidence/leakage/2026-10-31-x86_64.md` |
| Fuzz campaigns | `docs/FUZZ_EVIDENCE.md` |
| Mutation testing | `docs/MUTATION_EVIDENCE.md` |
| Release reproducibility | `evidence/release/2026-10-31-x86_64.md` |
| Hardware protocol | `docs/HARDWARE.md` |
| Release and signing | `docs/RELEASE.md` |
| Red-team program | `docs/RED_TEAM.md` |
| Local adversarial audit | `docs/AUDIT_REPORT_2026-10-30.md` |
| Tool inventory | `docs/TOOLING.md` |
| Local proof runner | `docs/LOCAL_PROOF.md` |
| Zeroization review | `docs/ZEROIZATION.md` |
| Known gaps | `docs/GAP_REPORT.md` |

Generate a self-contained bundle:

```bash
python3 tools/evidence_bundle.py --output evidence/bundle
```

## Questions for reviewers

1. Is the two-share masked adder correct and first-order secure under the stated
   probing model?
2. Does masked ChaCha20 compose the adder, XOR, and rotation gadgets without
   introducing first-order leakage?
3. Are the constant-time claims supported by the assembly checks and timing evidence?
4. Can compiler optimization break any masking barrier on a supported target?
5. Are the entropy adapter and failure paths fail-closed?
6. Can the trace parser, CLI, or synthetic generator be made to panic, hang, or
   exhaust memory?
7. Does the release process produce reproducible, traceable, signed artifacts?
8. Are the documented claims narrower than or equal to the evidence?

## Access

Provide the auditor with:

- the repository at the reviewed commit;
- the evidence bundle;
- the release manifest, SBOM, and provenance;
- the hardware protocol and target description;
- a private reporting channel.

## Constraints

The auditor must not test production systems or real user data. Use synthetic keys,
isolated hardware, and a written scope/authorization document.

## Current status

No independent audit or red-team engagement has been completed. This package exists to
make the review efficient once an external reviewer is engaged.
