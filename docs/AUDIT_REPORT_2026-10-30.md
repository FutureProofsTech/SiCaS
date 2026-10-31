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

### A-12 — Medium — Batch counter state desynchronized on final entropy failure

**Baseline behavior:** `MaskedChaCha20::keystream_blocks` advanced `self.counter` before
acquiring the final `mask_u32` used to refresh the scalar counter share. If that entropy
call failed, the batch returned an error but the counter had advanced while
`self.state[12]` still held the old counter.

**Impact:** A subsequent scalar `keystream_block` could emit a block with a stale
counter share after a failed batch. This is a fail-closed/state-consistency defect, not
a key-disclosure path.

**Fix:** acquire `state_mask` before mutating `self.counter` or `self.state[12]`. Added
regression tests for early entropy failure, final counter-mask failure, counter
wraparound, and scalar/batch interoperability.

**Evidence:** `crates/sicas-reference/src/chacha20.rs`; tests
`batch_counter_mask_failure_does_not_advance_state`,
`bitsliced_batch_handles_counter_wrap`, and `scalar_then_batch_is_consistent`.

**Residual risk:** None for this path after the fix. Hardware leakage remains an
external gate.

## Positive controls observed

- No `unsafe` in published crates.
- Zero third-party runtime dependencies.
- `Secret` deliberately lacks equality, indexing, and cloning.
- Constant-time assembly checks on four architectures, plus a 69-configuration
  optimization/codegen/panic/LTO matrix.
- x86_64 dudect evidence below threshold.
- Kani proofs for core gadgets and every masked operation, including addition,
  refresh, selection, equality, and share reconstruction.
- Fuzz campaigns, Miri, ASan, mutation, coverage, audit, deny, and reproducibility gates.
- Release manifest, SBOM, provenance, and Sigstore workflow are prepared.

## Verification after fixes

- Core: 20 tests.
- Harness: 91 library, 7 main, 5 CLI tests (at the full-proof commit).
- Platform: 4 tests.
- Reference: 15 tests; zeroization: 3 tests (at the full-proof commit).
- Doctests: 3 pass, 4 compile-fail.
- Miri: full workspace passes (core 20; harness 82 passed with 9 native-only
  statistical sweeps ignored; main 5 passed with 2 file/timing tests ignored; platform
  2 passed with 2 OS-entropy tests ignored; reference 7 passed with 8 batch/statistical sweeps
  ignored; 7 doctests pass). Ignored tests are blocked by Miri isolation or are
  exhaustive native sweeps; none are deleted.
- ASan: 133 workspace library tests pass.
- AArch64 QEMU: 133 workspace library tests pass.
- i686 native: 146 all-target tests pass.
- Kani: 30 harnesses pass.
- Fuzz: masked ops, lookup, trace parser, and ChaCha20 campaigns.
- Mutation: core 114 caught/0 missed/19 unviable; reference 69 caught/0 missed/8
  unviable; report-only workspace 544 caught/1 missed/55 unviable. The single
  workspace miss was the `ZeroizingBytes` drop path, which had no test at that
  commit; commit `92af463` added the drop test and the workspace rerun records zero
  missed viable mutants.
- Coverage: 97.23% lines, 94.82% regions.
- Audit, deny, reproducibility, and assembly evidence are recorded in the evidence
  bundle.
- Full local proof: all gates pass at commit `8c41e4b`, including the performance gate
  at 7.15×; see `evidence/local-proof-2026-10-31-final-audit/SUMMARY.md`.

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
- The assembly checker now covers 69 configurations: opt-level 0 on four targets,
  opt-levels 0/1/2/3/s and codegen/panic variants on x86_64 and AArch64, and x86_64
  thin/fat LTO.
- Valgrind Memcheck passes with zero errors on all four library test binaries at the
  full-proof commit; the zeroization binary was added later, and the current
  five-binary run also reports zero errors.
- Statistics were differentially validated against an independent Python model over
  200 deterministic cases.
- A local signing dry run produces and verifies a detached signature with an ephemeral
  test key; it is not a production identity.
- The scalar masked ChaCha20 measured ~600× overhead. A 64-block bitsliced batch path
  with a fast 64×64 transpose measured 7.15× vs unmasked in the full proof (7.26× in
  an earlier run). The 10× hard ceiling is met; the 5× target remains open in
  `docs/PERFORMANCE_EVIDENCE.md`.
- A first-order DPA difference-of-means tool was added to the harness with an AES
  S-box target, synthetic single-bit key-recovery tests, and a CLI `dpa` command.
- The experimental `sicas-zeroize` adapter now has a drop-path test, a volatile-write
  assembly check, Miri coverage of its safe paths, mutation coverage, and five-binary
  Valgrind Memcheck. It remains non-published and not integrated into the published
  crates.
- At commit `ef13675`, the workspace has 98 harness library tests, 8 harness main tests,
  and 5 harness CLI tests; ASan passes 141 workspace library tests, AArch64 QEMU passes
  141, i686 passes 154 all-target tests, and the report-only workspace mutation run has
  zero missed viable mutants.

These changes strengthen the local evidence. They do not change the external-only
gates: hardware leakage measurement, independent audit, independent red team,
zeroization adapter review, and production signing remain incomplete.

## Conclusion

No open critical or high-severity defect remains in the locally reviewable scope after
the fixes above. Every locally executable audit technique has been run: source review,
differential testing, fuzzing, mutation, Miri, ASan, Valgrind, Kani, assembly
inspection, leakage modeling, release validation, and self-audit.

The remaining risk is not unexamined code. It is independent verification and physical
measurement: an independent audit, an independent red team, real hardware power/EM
traces, an audited unsafe zeroization boundary, and a trusted production signing
identity. Self-audit cannot make itself independent, and software cannot produce
physical traces from hardware it does not have. Those are the only open items.
