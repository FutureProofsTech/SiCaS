# Gap Report

All local security gates that can be executed in this environment are implemented. The
masked-ChaCha20 hard performance ceiling is met; the 5× target remains open. The following gates are external,
physical, or require credentials and therefore cannot be truthfully completed here.

| Gate | Status | Why it remains | Required next action |
|---|---|---|---|
| Hardware power/EM TVLA/CPA/DPA | Blocked | No target board or acquisition hardware | Execute `docs/HARDWARE.md` on a Cortex-M4 target; publish raw traces |
| ARM/RISC-V runtime timing | Blocked | No physical targets; QEMU timing is not evidence | Run `sicas-harness ct` on each target; publish results |
| AArch64 functional runtime | Complete | QEMU user-mode workspace tests | See `docs/CROSS_ARCH_EVIDENCE.md` |
| External security audit | Not started | Requires an independent auditor | Engage an auditor using `docs/AUDIT_PACKAGE.md` |
| Independent red team | Not started | Requires an independent team and lab | Engage using `docs/RED_TEAM.md` |
| Release signing | Prepared | Requires a release identity and signing key/OIDC | Run `.github/workflows/release.yml` with Sigstore identity |
| SLSA provenance | Prepared | Requires a GitHub Actions run | Run the release workflow and publish the attestation |
| Production version claim | Not claimed | External gates above are incomplete | Keep the release labelled pre-production |
| Volatile zeroization | Experimental adapter implemented; not audited | `sicas-zeroize` uses volatile writes and a compiler fence, but the unsafe boundary needs independent review | Review `crates/sicas-zeroize`; see `docs/ZEROIZATION.md` |
| Full masked-ChaCha20 audit | Internal adversarial review only | No independent cryptographic review | Include it in the external audit scope; see `docs/AUDIT_REPORT_2026-10-30.md` |

## Local engineering gaps

| Gap | Status | Evidence | Required next action |
|---|---|---|---|
| Masked ChaCha20 performance | Hard ceiling met; target open | `docs/PERFORMANCE_EVIDENCE.md`: bitsliced batch is 7.15× vs unmasked in the final full proof (7.26× in an earlier run), so the 10× hard ceiling is met; the 5× target is not | Continue optimizing toward 5× with wider SIMD lanes or a faster audited mask stream |
| TVLA zero-variance false negative | Fixed 2026-11-05 | `stats::tests::welch_t_detects_a_shift`, `tvla::tests::constant_shift_with_zero_variance_is_detected`, `tools/check_stats.py` edge cases | None; keep the constant-group regression cases |
| Secret-dependent branch in bitsliced key extraction | Fixed 2026-11-05 | `chacha20::tests::masked_word_planes_extract_every_bit_without_a_share_branch`; `opt-level=0` review shows `setne` instead of a share-bit branch | Add the reference path to an assembly gate; currently source review plus regression test |
| Evidence bundle source completeness | Fixed 2026-11-05 | `tools/evidence_bundle.py` now includes all crate/fuzz/tool/workflow sources and rejects tracked working-tree modifications | Regenerate the bundle and review the new `INDEX.md` |
| Release tag/workflow binding | Fixed 2026-11-05 | `release.yml` checks out the requested tag, verifies the tag commit, and pins actions to SHAs; `tools/check_workflow_pins.py` is a local/CI gate | Run the release workflow with a real signing identity |
| Alignment normalization and shift-search DoS | Fixed 2026-11-05 | `preprocess::tests::correlation_is_normalized_and_amplitude_invariant`, `align_caps_an_oversized_shift_search_to_the_trace_length` | None |
| Synthetic generator allocation DoS | Fixed 2026-11-05 | `synthetic::tests::oversized_generation_is_rejected_before_allocation` and TVLA variant | None |
| Trace I/O syscall amplification | Fixed 2026-11-05 | Buffered reader/writer; `trace::tests::parser_buffers_many_small_samples` | Consider a streaming API for traces larger than the declared 100M-sample limit |
| TVLA performance target vs. trace limit | Documented 2026-11-05 | `PLAN.md` now states 1M traces × 100 samples (100M total); 1M × 1k would need a streaming format | Add a streaming trace format before a 1M × 1k campaign |
| Unbounded leakage/timing configuration | Fixed 2026-11-05 | `MAX_SAMPLES`, `MAX_TIMING_WARMUP`, `MAX_CT_TRIALS` and regression tests | None |

## Local readiness matrix

[`SAL4_LOCAL_READINESS.md`](SAL4_LOCAL_READINESS.md) maps every SAL-4 criterion to its
local evidence and records the remaining performance and zeroization gaps.

## Current label

`SAL-2 locally` — strong local verification and evidence, not a production security
claim.

## What is complete

- Safe Rust core, zero third-party runtime dependencies.
- Constant-time assembly checks on four architectures.
- x86_64 dudect timing evidence.
- 1M-execution fuzz campaigns on five targets.
- Full local proof at `8c41e4b`: all gates passed, including the performance gate at
  7.15×. The 10× hard ceiling is met; the 5× target remains open.
- Masked ChaCha20 RFC 8439 and differential validation.
- Compiler-matrix assembly evidence (69 configurations), Valgrind Memcheck with zero
  errors, differential statistics validation, release-profile timing, a long fuzz
  campaign, and a local signing dry run.
- Trace preprocessing, TVLA, CPA, timing, and synthetic validation.
- SBOM, checksums, provenance, and a release workflow.
- Threat model, claims, proof obligations, audit package, and red-team program.
- Internal red-team review on 2026-11-05 with PoCs and fixes for a secret-dependent
  branch, TVLA zero-variance false negative, release-tag binding, evidence-bundle
  completeness, alignment/DoS, synthetic allocation, trace I/O, and configuration
  bounds. This is not an independent review; see
  [`RED_TEAM_REPORT_2026-11-05.md`](RED_TEAM_REPORT_2026-11-05.md).
