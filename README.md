# SiCaS

Safe, zero-dependency Rust primitives for side-channel attenuation and leakage
assessment.

## Status

The full local proof is at commit `8c41e4b`; the AES S-box DPA addition was verified by
a quick proof at `ef13675`. The bitsliced masked-ChaCha20 batch meets the 10×
performance hard ceiling at 7.15× in the final full proof; the 5× target is open. The
project is **not audited, not hardware-validated, and must not be used to protect real
secrets yet**. The current public label is **SAL-2 locally**. The
production target is SAL-4 as defined in [`PLAN.md`](PLAN.md) and
[`docs/ASSURANCE.md`](docs/ASSURANCE.md).

Project operating instructions and persistent working agreement:
[`AGENTS.md`](AGENTS.md) and [`docs/WORKING_AGREEMENT.md`](docs/WORKING_AGREEMENT.md).
External audit is the final second opinion; the local proof is complete first.

## What exists

- `sicas-core`: `no_std`, `#![forbid(unsafe_code)]`, zero runtime dependencies.
  - `Choice` and constant-time comparison/selection/swap/lookup for
    `u8`, `u16`, `u32`, `u64`, and `u128`.
  - `Secret<T>` with redacted `Debug` and no accidental equality/indexing/cloning.
  - `Masked<T>` with two-share first-order Boolean masking for
    `u8`, `u16`, `u32`, `u64`, and `u128`.
  - `EntropySource` with fail-closed behavior.
- `sicas-harness`: `std`, zero third-party dependencies.
  - Strict binary trace format with typed errors and a fuzzed parser.
  - Trace preprocessing: trim, moving-average filter, and cross-correlation alignment.
  - Fixed-vs-random TVLA with Welch's t-test.
  - Byte-oriented CPA with Hamming-weight and custom leakage models.
  - Deterministic synthetic trace generation.
  - Dudect-style timing tooling.
  - CLI: `tvla`, `cpa`, `dpa`, `synthetic`, `synthetic-cpa`, `timing`, `ct`, `trim`,
    `filter`, `align`, `leakage-check`.
- `sicas-reference`: `no_std`, zero third-party dependencies.
  - First-order Boolean masked ChaCha20 keystream blocks.
  - RFC 8439 vector and differential tests against an unmasked reference.
- `sicas-zeroize`: experimental, non-published volatile zeroization boundary; unaudited.
- `sicas-platform`: safe, zero third-party dependencies.
  - Fail-closed OS entropy adapter for Unix-like systems.
  - Unsupported platforms return `EntropyError::Unavailable`.
- Unit tests, exhaustive byte tests, property-style randomized tests, compile-fail
  doctests, Kani proof harnesses, and libFuzzer targets.
- AddressSanitizer, Miri, strict formatting, and strict Clippy gates.
- Dudect-style constant-time evidence suite and assembly inspection target.

See:

- [`docs/EVALUATION.md`](docs/EVALUATION.md) — trace format and harness usage
- [`docs/HARDWARE.md`](docs/HARDWARE.md) — hardware evidence protocol
- [`docs/CT_EVIDENCE.md`](docs/CT_EVIDENCE.md) — constant-time timing evidence
- [`docs/CROSS_ARCH_EVIDENCE.md`](docs/CROSS_ARCH_EVIDENCE.md) — AArch64 QEMU functional evidence
- [`docs/FUZZ_EVIDENCE.md`](docs/FUZZ_EVIDENCE.md) — 1M-execution fuzz campaigns
- [`docs/MUTATION_EVIDENCE.md`](docs/MUTATION_EVIDENCE.md) — mutation-testing results
- [`evidence/leakage/2026-10-31-x86_64.md`](evidence/leakage/2026-10-31-x86_64.md) — first-order leakage-model evidence
- [`docs/PERFORMANCE_EVIDENCE.md`](docs/PERFORMANCE_EVIDENCE.md) — masked ChaCha20 performance and open gap
- [`docs/RELEASE.md`](docs/RELEASE.md) — release, SBOM, and signing
- [`docs/RED_TEAM.md`](docs/RED_TEAM.md) — independent review program
- [`PRODUCTION_READINESS.md`](PRODUCTION_READINESS.md) — production status and blockers
- [`AUDIT_REQUEST.md`](AUDIT_REQUEST.md) — formal independent review request
- [`AUDITOR_GUIDE.md`](AUDITOR_GUIDE.md) — auditor quick start and claim map
- [`docs/AUDITOR_CHECKLIST.md`](docs/AUDITOR_CHECKLIST.md) — reproducible review checklist
- [`docs/TEST_MATRIX.md`](docs/TEST_MATRIX.md) — property-to-test map
- [`docs/AUDIT_TECHNIQUE_MATRIX.md`](docs/AUDIT_TECHNIQUE_MATRIX.md) — auditor technique-to-evidence map
- [`docs/AUDIT_PACKAGE.md`](docs/AUDIT_PACKAGE.md) — audit scope and evidence index
- [`docs/AUDIT_REPORT_2026-10-30.md`](docs/AUDIT_REPORT_2026-10-30.md) — adversarial review findings and fixes
- [`docs/SELF_AUDIT_2026-10-31.md`](docs/SELF_AUDIT_2026-10-31.md) — current self-audit and fix record
- [`docs/GAP_REPORT.md`](docs/GAP_REPORT.md) — remaining external gates
- [`docs/LOCAL_PROOF.md`](docs/LOCAL_PROOF.md) — one-command local proof runner
- [`docs/POST_PROOF_BACKLOG.md`](docs/POST_PROOF_BACKLOG.md) — work after local proof, before external review
- [`docs/SAL4_LOCAL_READINESS.md`](docs/SAL4_LOCAL_READINESS.md) — local SAL-4 evidence matrix
- [`docs/SAL4_EXECUTION_PLAN.md`](docs/SAL4_EXECUTION_PLAN.md) — SAL-4 criteria and execution status
- [`docs/BITSLICED_MASKING.md`](docs/BITSLICED_MASKING.md) — bitsliced masking security argument
- [`docs/ZEROIZATION.md`](docs/ZEROIZATION.md) — zeroization limitation and adapter review checklist

## Quality gates

```bash
./tools/local_proof.sh quick   # fast gate set
./tools/local_proof.sh full    # full local proof, 20+ minutes
```

The full run covers formatting, strict Clippy, all tests and doctests, full-workspace
Miri, Kani, ASan, mutation testing, coverage, five 1M-execution fuzz campaigns,
cross-architecture tests, leakage-model checks, release validation, and reproducible
builds. See [`docs/LOCAL_PROOF.md`](docs/LOCAL_PROOF.md).

## Constraints

- Safe Rust: `sicas-core` forbids unsafe code.
- Zero runtime dependencies: no third-party crates.
- The non-published `fuzz/` crate uses `libfuzzer-sys`; it is excluded from the workspace
  and is never shipped.
- Every security claim must link to reproducible evidence. See
  [`docs/SECURITY_CLAIMS.md`](docs/SECURITY_CLAIMS.md) and
  [`docs/PROOF_OBLIGATIONS.md`](docs/PROOF_OBLIGATIONS.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT License ([`LICENSE-MIT`](LICENSE-MIT))

at your option.
