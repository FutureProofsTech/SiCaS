# Assurance Program

## Current level

**SAL-2 locally** (tests, CI, Miri, sanitizers, fuzzing, static analysis). The full
local proof is at `8c41e4b`; the AES S-box DPA addition was verified by a quick proof at
`ef13675`. The bitsliced masked-ChaCha20 batch meets the 10× performance hard ceiling at
7.15× in the final full proof; the 5× target is open. The target for
v1.0 is **SAL-4**. CI, hardware evidence, independent audit, and red team are still
pending.

| Level | Requirements | Allowed public claim |
|-------|--------------|----------------------|
| SAL-0 | Demo or prototype only | "experimental" |
| SAL-1 | Unit/integration tests + CI | "tested" |
| SAL-2 | SAL-1 + Miri, sanitizers, fuzzing, static analysis | "hardened" |
| SAL-3 | SAL-2 + formal/model checking of critical invariants + real-hardware TVLA + external audit | "audited" |
| SAL-4 | SAL-3 + independent red team + reproducible/signed builds + no open critical/high findings + public evidence bundle | "SAL-4 under threat model T" |

## Gates

| Gate | Frequency | Required checks |
|------|-----------|-----------------|
| PR | Every push | fmt, strict Clippy, unit/integration/doc tests, compile-fail, Miri, Kani smoke, fuzz smoke, cross-target check, zero-dependency check |
| Nightly | Daily | Full fuzzing, sanitizers, mutation testing, coverage, Kani full suite, feature powerset, reproducible build, Valgrind |
| Weekly | Weekly | Hardware TVLA/CPA, compiler matrix, platform matrix, red-team regression |
| Release | Every release | SAL-4 evidence bundle, audit/red-team sign-off, signed artifacts, SBOM, reproducible build verification |

## Evidence bundle

- Threat model, claims, non-claims.
- Test, coverage, mutation, and fuzz reports.
- Kani/model-checking proofs and assumptions.
- Constant-time assembly and dudect reports.
- First-order leakage-model checks with a negative control.
- Compiler-matrix assembly checks and Valgrind Memcheck reports.
- Differential statistics validation and a local signing dry run.
- Performance measurements and the documented masked-ChaCha20 gap.
- TVLA/CPA/DPA raw traces and analysis scripts.
- Red-team and audit reports, remediation, retest.
- Local adversarial audit report and remediation record.
- SBOM, build hashes, signatures, and reproduction commands.
- Release manifest, provenance, and signing verification.
- Known limitations and accepted risks.

A release may claim SAL-4 only when every criterion is met and independently verified.
