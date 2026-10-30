# Audit Report — 2026-10-30

**Reviewer:** Adversarial external-auditor simulation  
**Baseline reviewed:** `9da7463`  
**Fixes:** `45a8cce` and the commit containing this report  
**Scope:** `sicas-core`, `sicas-harness`, `sicas-reference`, `sicas-platform`, build/CI/release

## Independence caveat

This is an adversarial self-review performed locally. It is not an independent
third-party audit. It is useful for finding defects before an external review; it
does not satisfy the independent-audit gate.

## Findings

### A-01 — High — Correlated shares in masked ChaCha20 final addition

**Baseline behavior:** `MaskedChaCha20::keystream_block` created the `initial` state by
cloning the working state. The final ChaCha20 addition therefore fed two representations
with identical masks into the masked adder. Functional tests pass, but the first-order
masking assumption is not justified.

**Impact:** A first-order leakage claim for masked ChaCha20 would be unsupported. The
adder may still be functionally correct, but the composition is not a valid
independently-masked addition.

**Fix:** Store a separately masked `initial` state with independent masks. Key words and
the counter are masked twice with independent entropy. The working state is derived from
one representation; the final addition uses the other. Counter updates refresh both
representations.

**Evidence:** `crates/sicas-reference/src/chacha20.rs`; RFC 8439 and differential tests
still pass.

**Residual risk:** This is a design correction, not a formal leakage proof. Hardware
TVLA remains required.

### A-02 — High — Integer overflow in `moving_average`

**Baseline behavior:** The moving-average sum used `i32`. A window over more than about
65,536 samples of large amplitude could overflow, causing a debug panic or a wrapped
result in release.

**Impact:** Incorrect analysis or denial of service on large traces.

**Fix:** Use `i64` prefix sums, which also reduces the filter from O(n·window) to O(n).
A 100,000-sample regression test covers the previous overflow boundary.

**Evidence:** `crates/sicas-harness/src/preprocess.rs`.

### A-03 — Medium — Empty trace serialization was inconsistent

**Baseline behavior:** `TraceSet::split_at` can produce an empty side. `write_to` wrote a
zero trace count, but `read_from` rejects zero dimensions. The resulting file could not
be read back.

**Impact:** Confusing or corrupt analysis artifacts; no memory-safety issue.

**Fix:** `write_to` rejects empty sets with `InvalidDimensions`; `split_at` documents that
empty sides are analysis-only. A regression test covers the behavior.

**Evidence:** `crates/sicas-harness/src/trace.rs`.

### A-04 — Medium — Masked-adder entropy contract was under-documented

**Baseline behavior:** `Masked::add_masked` required fresh independent masks for every
masked AND, but the public documentation did not state the entropy-source requirement
explicitly.

**Impact:** A caller could pass a predictable or reused source and invalidate the
masking assumption while believing the API was safe.

**Fix:** Added a `# Security` section requiring fresh, independent masks.

**Evidence:** `crates/sicas-core/src/masked.rs`.

### A-05 — Low — Cloning `Masked` duplicates shares

**Observation:** `Masked` is `Clone`. Cloning does not re-randomize shares.

**Disposition:** Documented in the type docs. This is intentional for composition and
does not combine shares. Callers must not treat a clone as a fresh mask.

### A-06 — Low — Empty split sides cannot be serialized

**Observation:** `TraceSet::split_at` can return an empty side, which is useful for TVLA
control groups but not serializable.

**Disposition:** Documented; `write_to` now rejects it. Analysis APIs handle empty groups
explicitly.

### A-07 — Accepted — No volatile zeroization

**Observation:** `Secret` and `Masked` do not guarantee volatile erasure on drop. Safe
Rust cannot provide that guarantee without an audited unsafe boundary.

**Disposition:** Documented limitation. A platform zeroization adapter remains an
external design/review item.

### A-08 — Accepted — Release workflow not exercised on GitHub

**Observation:** The Sigstore/SLSA release workflow is locally reviewed and the manifest
path is tested, but the GitHub OIDC signing run has not executed.

**Disposition:** Documented in `docs/RELEASE.md`; requires a real release run.

### A-09 — Accepted — External/hardware gates remain

**Observation:** No hardware TVLA/CPA/DPA, independent red team, or external audit has
been performed.

**Disposition:** Recorded in `docs/GAP_REPORT.md`; no production claim is made.

## Positive controls observed

- No `unsafe` in published crates.
- Zero third-party runtime dependencies.
- `Secret` deliberately lacks equality, indexing, and cloning.
- Constant-time assembly checks on four architectures.
- x86_64 dudect evidence below threshold.
- Kani proofs for core gadgets and masked addition.
- Fuzz campaigns, Miri, ASan, mutation, coverage, audit, deny, and reproducibility gates.
- Release manifest, SBOM, provenance, and Sigstore workflow are prepared.

## Verification after fixes

- Core: 18 tests.
- Harness: 63 library, 6 main, 5 CLI tests.
- Platform: 4 tests.
- Reference: 6 tests.
- Doctests: 3 pass, 4 compile-fail.
- Miri: core, platform, and reference pass (OS entropy tests ignored under Miri
  isolation); the harness re-run is part of this audit evidence.
- ASan: 91 workspace library tests pass.
- Kani: 14 harnesses pass.
- Fuzz: masked ops, lookup, trace parser, and ChaCha20 campaigns.
- Mutation: core 111 caught/0 missed; reference 10 caught/0 missed; workspace 405
  caught, 21 missed, 36 unviable (95.1% of viable mutants).
- Coverage: 96.50% lines, 93.87% regions.
- Audit, deny, reproducibility, and assembly evidence are recorded in the evidence
  bundle.

## Conclusion

No open critical or high-severity defect remains in the locally reviewable scope after
the fixes above. The remaining risk is concentrated in physical leakage validation,
independent review, zeroization, and release signing — none of which can be completed
truthfully on this local machine alone.
