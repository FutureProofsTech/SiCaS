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

### A-09 — Medium — Timing harness could be constant-folded

**Baseline behavior:** the `ct` suite used constant masks and a constant lookup table.
In release builds the compiler could optimize the measured work, producing unrealistically
small timings (observed on i686).

**Impact:** timing evidence could be misleading even when the implementation is
constant-time.

**Fix:** added `black_box` barriers around entropy bytes, mask constants, the lookup
table, and operation inputs. Re-ran the x86_64 5×1M suite; maximum absolute t is now
2.2472, still below the 4.5 threshold.

**Residual risk:** i686 timing remains unused because 32-bit timer resolution and
optimization make it unreliable; i686 functional tests are retained.

### A-10 — Medium — Fixed masks reused across ChaCha20 blocks

**Baseline behavior:** after the A-01 fix, the key and initial state were independently
masked, but the same initial-state shares were reused for every block.

**Impact:** Reusing a fixed mask across multiple blocks weakens the multi-block
first-order masking argument.

**Fix:** added `Masked::refresh` and refresh both the working and initial state with
independent fresh masks at the start of every block. The stored key shares are never
used directly in a round; they are refreshed first.

**Evidence:** `crates/sicas-core/src/masked.rs`,
`crates/sicas-reference/src/chacha20.rs`, RFC 8439 and differential tests.

**Residual risk:** This is still a design argument, not a formal multi-block leakage
proof. Hardware TVLA remains required.

### A-11 — Accepted — External/hardware gates remain

**Observation:** No hardware TVLA/CPA/DPA, independent red team, or external audit has
been performed.

**Disposition:** Recorded in `docs/GAP_REPORT.md`; no production claim is made.

## Positive controls observed

- No `unsafe` in published crates.
- Zero third-party runtime dependencies.
- `Secret` deliberately lacks equality, indexing, and cloning.
- Constant-time assembly checks on four architectures, plus a 60-configuration
  optimization/codegen/panic/LTO matrix.
- x86_64 dudect evidence below threshold.
- Kani proofs for core gadgets and every masked operation, including addition,
  refresh, selection, equality, and share reconstruction.
- Fuzz campaigns, Miri, ASan, mutation, coverage, audit, deny, and reproducibility gates.
- Release manifest, SBOM, provenance, and Sigstore workflow are prepared.

## Verification after fixes

- Core: 20 tests.
- Harness: 91 library, 7 main, 5 CLI tests.
- Platform: 4 tests.
- Reference: 11 tests.
- Doctests: 3 pass, 4 compile-fail.
- Miri: full workspace passes (core 20; harness 82 passed with 9 native-only
  statistical sweeps ignored; main 5 passed with 2 file/timing tests ignored; platform
  2 passed with 2 OS-entropy tests ignored; reference 7 passed with 4 batch/statistical sweeps
  ignored; 7 doctests pass). Ignored tests are blocked by Miri isolation or are
  exhaustive native sweeps; none are deleted.
- ASan: 126 workspace library tests pass.
- AArch64 QEMU: 126 workspace library tests pass.
- i686 native: 138 all-target tests pass.
- Kani: 30 harnesses pass.
- Fuzz: masked ops, lookup, trace parser, and ChaCha20 campaigns.
- Mutation: core 112 caught/0 missed; reference 69 caught/0 missed; workspace 542
  caught, 0 missed, 51 unviable (100% of viable mutants).
- Coverage: 97.11% lines, 94.66% regions.
- Audit, deny, reproducibility, and assembly evidence are recorded in the evidence
  bundle.
- Full local proof: all gates pass at commit `ac06c1c`, including the performance gate
  at 9.17×; see `evidence/local-proof-2026-10-31-bitsliced/SUMMARY.md`.

## Post-proof hardening (2026-10-31)

Additional local hardening was completed after the initial audit pass:

- Kani coverage was extended to every masked linear and logical operation across
  `u8`, `u16`, `u32`, `u64`, and `u128`, plus bit extraction.
- `Masked::shares` was added for explicit leakage-analysis boundaries, with a security
  warning.
- A deterministic first-order leakage-model checker now covers masked bytes, masked
  AND, masked addition, and masked ChaCha20 key shares, with a negative control that
  must detect unmasked leakage.
- Property tests were added for trace trimming, moving averages, alignment shifts and
  tie-breaking, CPA key recovery, and TVLA tie/threshold behavior.
- Timing validation was made explicit at the four-sample boundary; duration conversion
  is unit-tested.
- A release-manifest validator checks artifact hashes, provenance, SBOM components and
  dependencies, workspace versions, timestamps, and the current commit.
- A zeroization design review records the limitation and the requirements for a future
  audited adapter.
- The assembly checker now covers 60 configurations: opt-level 0 on four targets,
  opt-levels 0/1/2/3/s and codegen/panic variants on x86_64 and AArch64, and x86_64
  thin/fat LTO.
- Valgrind Memcheck passes with zero errors on all four library test binaries.
- Statistics were differentially validated against an independent Python model over
  200 deterministic cases.
- A local signing dry run produces and verifies a detached signature with an ephemeral
  test key; it is not a production identity.
- The scalar masked ChaCha20 measured ~600× overhead. A 64-block bitsliced batch path
  with a fast 64×64 transpose now measures 9.17× vs unmasked. The 10× hard ceiling
  is met; the 5× target remains open in `docs/PERFORMANCE_EVIDENCE.md`.

These changes strengthen the local evidence. They do not change the external-only
gates: hardware leakage measurement, independent audit, independent red team,
zeroization adapter review, and production signing remain incomplete.

## Conclusion

No open critical or high-severity defect remains in the locally reviewable scope after
the fixes above. The remaining risk is concentrated in physical leakage validation,
independent review, zeroization, and release signing — none of which can be completed
truthfully on this local machine alone.
