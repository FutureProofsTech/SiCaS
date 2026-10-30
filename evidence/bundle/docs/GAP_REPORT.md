# Gap Report

All local engineering gates that can be executed in this environment are implemented.
The following gates are external, physical, or require credentials and therefore cannot
be truthfully completed here.

| Gate | Status | Why it remains | Required next action |
|---|---|---|---|
| Hardware power/EM TVLA/CPA/DPA | Blocked | No target board or acquisition hardware | Execute `docs/HARDWARE.md` on a Cortex-M4 target; publish raw traces |
| ARM/RISC-V runtime timing | Blocked | No physical targets; QEMU timing is not evidence | Run `sicas-harness ct` on each target; publish results |
| External security audit | Not started | Requires an independent auditor | Engage an auditor using `docs/AUDIT_PACKAGE.md` |
| Independent red team | Not started | Requires an independent team and lab | Engage using `docs/RED_TEAM.md` |
| Release signing | Prepared | Requires a release identity and signing key/OIDC | Run `.github/workflows/release.yml` with Sigstore identity |
| SLSA provenance | Prepared | Requires a GitHub Actions run | Run the release workflow and publish the attestation |
| Production version claim | Not claimed | External gates above are incomplete | Keep the release labelled pre-production |
| Volatile zeroization | Not guaranteed | Safe Rust cannot guarantee volatile erasure without an audited unsafe boundary | Design and review an isolated zeroization adapter |
| Full masked-ChaCha20 audit | Internal adversarial review only | No independent cryptographic review | Include it in the external audit scope; see `docs/AUDIT_REPORT_2026-10-30.md` |

## Current label

`SAL-2 locally` — strong local verification and evidence, not a production security
claim.

## What is complete

- Safe Rust core, zero third-party runtime dependencies.
- Constant-time assembly checks on four architectures.
- x86_64 dudect timing evidence.
- 1M-execution fuzz campaigns on four targets.
- Miri, ASan, Kani, mutation, coverage, audit, deny, and reproducibility gates.
- Masked ChaCha20 RFC 8439 and differential validation.
- Trace preprocessing, TVLA, CPA, timing, and synthetic validation.
- SBOM, checksums, provenance, and a release workflow.
- Threat model, claims, proof obligations, audit package, and red-team program.
