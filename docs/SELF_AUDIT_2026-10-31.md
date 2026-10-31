# Self-Audit — 2026-10-31

This is an internal adversarial review of the current implementation. It is not an
independent audit and does not support a SAL-4 claim.

## Scope

- bitsliced masked ChaCha20 batch path;
- scalar masked ChaCha20 path;
- release profile (`opt-level=z`, fat LTO, panic=abort, stripped symbols);
- entropy adapters and failure paths;
- trace parser, preprocessing, TVLA, CPA, timing, leakage model;
- release manifest, SBOM, provenance, and evidence bundle.

## Method

- line-by-line review of the batch state machine and masked adder;
- differential tests against an independent unmasked ChaCha20 reference;
- lane-independent addition tests over 64 lanes;
- counter wraparound and scalar/batch interoperability tests;
- entropy-failure injection at early and final mask acquisition;
- mutation testing, Miri, ASan, Valgrind, five fuzz campaigns including the
  64-block bitsliced batch, and assembly inspection;
- statistical leakage checks with a negative control.

## Findings and fixes

### A-12 — Medium — Batch counter state desynchronized on final entropy failure

`keystream_blocks` advanced the counter before acquiring the final counter mask. If that
entropy call failed, the counter advanced while the scalar counter share stayed stale.
Fixed by acquiring the mask before mutating state. Regression tests cover early failure,
final counter-mask failure, counter wrap, and scalar/batch interoperability.

### A-01 to A-11

Recorded in `docs/AUDIT_REPORT_2026-10-30.md`: correlated ChaCha20 shares, moving-average
overflow, empty-trace serialization, timing constant folding, fixed-mask reuse, and the
external-only gates.

## Positive results

- Full local proof passes at the audited commit.
- Reference mutation has zero missed viable mutants.
- Assembly matrix covers 69 configurations, including `opt-level=z` and fat LTO.
- Bitsliced batch meets the 10× performance hard ceiling.
- Bitsliced key planes are statistically balanced across fresh masks.
- The dedicated batch fuzz target has a 1,000,000-execution campaign.
- Release binary is reproducible and smaller than the previous profile.

## Residual risk

- No hardware power/EM TVLA/CPA/DPA.
- No independent audit or red team.
- No production signing identity or SLSA attestation.
- Zeroization has an experimental volatile-write adapter with tests and x86_64
  assembly evidence; independent review is pending.
- The 5× performance target is open.
