# Red-Team and Deep Security Audit — 2026-11-05

**Reviewer:** internal adversarial review (not an independent red team)  
**Baseline:** `51cb12b` plus the fixes in this review  
**Public label at review time:** `SAL-2 locally`  
**Independence:** this is a local red-team simulation. It does **not** satisfy the
independent-red-team, external-audit, hardware, or production-signing gates. No
SAL-3/SAL-4 claim is made or implied.

## Scope and method

Reviewed:

- `sicas-core`, `sicas-reference`, `sicas-harness`, `sicas-platform`, and
  `sicas-zeroize`;
- the CLI, trace parser, preprocessing, TVLA, CPA, DPA, timing, leakage model, and
  synthetic generator;
- build, CI, release, SBOM, provenance, evidence-bundle, and signing tooling;
- the documented threat model, claims, proof obligations, and audit package.

Methods:

- line-by-line review with adversarial data-flow tracing;
- source-level search for secret-dependent branches, indexes, and error paths;
- `opt-level=0` assembly inspection of the bitsliced reference path (which the
  existing assembly matrix did not cover);
- hand-built malformed and pathological trace/config inputs;
- release-workflow and evidence-bundle integrity review;
- regression tests for every fixed finding.

## Findings

| ID | Severity | Finding | Status |
|---|---|---|---|
| RT-01 | High | Secret-dependent branches in bitsliced key-plane extraction | Fixed |
| RT-02 | High | TVLA/Welch false negative for zero-variance groups | Fixed |
| RT-03 | High | Release workflow could publish a tag that was not built | Fixed |
| RT-04 | High | Evidence bundle omitted the source it claimed to attest | Fixed |
| RT-05 | Medium | `align` was not normalized and could hang on a large shift range | Fixed |
| RT-06 | Medium | Synthetic generator allocated before dimension validation | Fixed |
| RT-07 | Medium | Trace I/O used one syscall per two-byte sample | Fixed |
| RT-08 | Medium | Unbounded leakage/timing configuration could exhaust memory or hang | Fixed |
| RT-09 | Medium | Constant-time claim was broader than the assembly evidence | Partially fixed / documented |
| RT-10 | Medium | CI and release actions were not pinned to commit SHAs | Fixed |
| RT-11 | Medium | Evidence tools could inspect stale artifacts | Fixed |
| RT-12 | Low | Original vision file contained superseded overclaims and a license mismatch | Fixed (archived) |
| RT-13 | Low | TVLA performance target exceeded the trace-format limit | Fixed (documented) |
| RT-14 | High | Release tag name was interpolated into a privileged shell script | Fixed |
| RT-15 | Low | Long-fuzz cache could reuse stale evidence | Fixed |
| RT-16 | Low | CI fuzz job omitted the bitsliced batch target | Fixed |

---

### RT-01 — High — Secret-dependent branches in bitsliced key-plane extraction

**Affected code:** `crates/sicas-reference/src/chacha20.rs`,
`masked_word_planes`.

**Observation.** The function converted each bit of the two key shares into an
all-ones or all-zero `u64` with:

```rust
let share0_bits = if (*share0 >> shift) & 1 == 1 { u64::MAX } else { 0 };
let share1_bits = if (*share1 >> shift) & 1 == 1 { u64::MAX } else { 0 };
```

Both conditions depend on a secret share bit. This violates the project rule that
security-critical code has no secret-dependent branch. It also bypassed the
existing assembly matrix, which inspects only `ct::eq_u32`, `ct::select_u32`, and
`ct::is_zero_u32`.

**PoC.** Build the reference benchmark example at `opt-level=0` and disassemble the
monomorphized `masked_word_planes` symbol:

```bash
CARGO_TARGET_DIR=target/redteam-o0 \
  cargo rustc -p sicas-reference --example benchmark --locked \
  -- --emit=asm -C opt-level=0
asm=$(ls -t target/redteam-o0/debug/examples/benchmark-*.s | head -1)
grep -n -A8 -B8 'masked_word_planes' "$asm"
```

On the baseline, the function body contains conditional jumps on the extracted
share bit (for example `je` immediately after the shift/and/test sequence). At
`opt-level=3` x86_64 the compiler happened to lower the selection to `setne`/`sbb`,
but that is an optimizer outcome, not a language guarantee, and it is not true on
all targets or optimization levels.

**Impact.** A key-share bit can influence control flow and therefore timing,
branch-predictor state, and instruction-cache behavior. This directly undermines
the constant-time part of the first-order masking argument for the bitsliced
reference path (C2/C3 evidence). It is not a demonstrated key-recovery attack, but
it is a genuine secret-dependent control-flow defect.

**Fix.** Bit extraction now uses a one-hot mask and branchless spreading:

```rust
let share0_bits = u64::from(u64::from(*share0) & bit_mask != 0).wrapping_neg();
let share1_bits = u64::from(u64::from(*share1) & bit_mask != 0).wrapping_neg();
```

`u64::from(bool)` is a `setcc`-style selection; `wrapping_neg` maps 0/1 to
0/`u64::MAX`. The new regression test
`masked_word_planes_extract_every_bit_without_a_share_branch` checks every bit
position for several words. A post-fix `opt-level=0` review shows `setne` at the
extraction site and no branch on the share bit.

**Residual risk.** The bitsliced reference path is still not part of the automated
assembly matrix. A dedicated reference assembly gate is a hardening item; until it
exists, the evidence is source review plus the regression test. This is recorded in
RT-09 and `docs/GAP_REPORT.md`.

---

### RT-02 — High — TVLA/Welch false negative for zero-variance groups

**Affected code:** `crates/sicas-harness/src/stats.rs` (`welch_t`,
`welch_t_from_sums`) and `tools/check_stats.py`.

**Observation.** When both groups had zero variance, the code returned `0.0`
regardless of the means:

```rust
if denominator <= 0.0 {
    return Some(0.0);
}
```

For two deterministic groups with different means, the Welch statistic is
infinite, not zero. This is the strongest possible leak: every trace in one group
is distinguishable from every trace in the other. The behavior made TVLA fail open.

**PoC.** Create two one-sample trace sets, one constant at 100 and one constant at
0, and run TVLA:

```bash
python3 - <<'PY'
import struct
def write(path, values):
    with open(path, "wb") as f:
        f.write(b"SiCaSTR1")
        f.write(struct.pack("<HII", 1, 1, len(values)))
        f.write(struct.pack("<" + "h" * len(values), *values))
write("/tmp/sicas-fixed.bin", [100, 100])
write("/tmp/sicas-random.bin", [0, 0])
PY
cargo run -q -p sicas-harness -- tvla \
  --fixed /tmp/sicas-fixed.bin --random /tmp/sicas-random.bin
```

On the baseline, the command prints `max |t| = 0.0000` and exits successfully. A
leaking target passes. The existing `welch_t_detects_a_shift` unit test encoded the
same incorrect expectation.

**Impact.** A deterministic or strongly quantized leak can be reported as clean.
This undermines C3 evidence and any future hardware TVLA result that contains
zero-variance samples. The independent Python differential model had the same
zero-variance convention, so it could not catch the defect.

**Fix.** The zero-denominator case now returns:

- `0.0` when the means are equal (identical constant groups);
- `+inf` or `-inf` when the means differ (deterministic leakage).

`tvla` treats an infinite `|t|` as leakage. Tests were updated and added:
`welch_t_detects_a_shift`,
`welch_t_from_sums_detects_zero_variance_leakage`, and
`tvla::tests::constant_shift_with_zero_variance_is_detected`. The Python model was
corrected and now has four deterministic edge cases (204 cases total), including
constant groups with equal and different means.

**Residual risk.** TVLA remains a statistical test with false positives and false
negatives on noisy data; this fix removes one specific fail-open case, not all of
them.

---

### RT-03 — High — Release workflow could publish a tag that was not built

**Affected file:** `.github/workflows/release.yml`.

**Observation.** The workflow supported `workflow_dispatch` with a `tag` input. The
checkout step used the default checkout ref (the branch selected when dispatching),
while the release step used the user-supplied `tag` as `tag_name`. A manual dispatch
could therefore build the default branch and publish a draft release named `vX`
without verifying that `vX` pointed at the built commit. There was no tag/commit
check.

**Impact.** A release artifact and its provenance could be attributed to a tag that
does not contain that code. This is a release-integrity and supply-chain defect.

**Fix.** The checkout now uses:

```yaml
with:
  ref: ${{ inputs.tag || github.ref }}
  fetch-depth: 0
```

and a verification step requires the tag to resolve to `HEAD`:

```yaml
- name: Verify the release tag points at the checked-out commit
  run: |
    tag="${{ inputs.tag || github.ref_name }}"
    test -n "$tag"
    test "$(git rev-parse "refs/tags/${tag}^{commit}")" = "$(git rev-parse HEAD)"
```

The release manifest also records and validates a `dirty` flag; a release build
from tracked modifications is rejected unless `--allow-dirty` is explicitly used.
The production signing run remains an external gate.

---

### RT-04 — High — Evidence bundle omitted the source it claimed to attest

**Affected file:** `tools/evidence_bundle.py`.

**Observation.** The README and audit guide described the bundle as containing a
“source snapshot for the code commit under test,” but the required-file list
contained only selected files. It omitted:

- `crates/sicas-core/src/*.rs`;
- `crates/sicas-harness/src/*.rs` except `dpa.rs`;
- `crates/sicas-reference/src/chacha20.rs`;
- `crates/sicas-platform/src/entropy.rs`;
- most fuzz targets and several tools;
- `.github/workflows/*.yml`.

A reviewer could not rebuild the reviewed code from the bundle. The bundle also
recorded `git rev-parse HEAD` without checking whether tracked files were modified,
so it could claim a commit while copying a dirty working tree.

**Impact.** The evidence bundle was not self-contained and was not cryptographically
or procedurally bound to a clean commit. This is an evidence-integrity defect, not a
runtime vulnerability.

**Fix.** `evidence_bundle.py` now:

- includes all `crates/**/*.rs`, `crates/**/Cargo.toml`, fuzz targets/seeds, tools,
  workflow files, and `docs/**/*.md` via required globs;
- fails if a required glob has no matches;
- checks `git status --porcelain --untracked-files=no` and refuses a dirty tracked
  tree unless `--allow-dirty` is passed;
- refuses a `--commit` value that differs from `HEAD` unless
  `--allow-different-commit` is passed;
- records the dirty state in `INDEX.md`.

The bundle still is not signed; production signing and SLSA provenance remain
external gates.

---

### RT-05 — Medium — `align` was not normalized and could hang on a large shift range

**Affected code:** `crates/sicas-harness/src/preprocess.rs`.

**Observation.** The documentation and README claimed “normalized
cross-correlation,” but `correlation` computed `sum(reference * trace) / overlap`.
That score is sensitive to amplitude and DC offset, so a high-amplitude or offset
trace can win the alignment search for the wrong shift. In addition, `align` looped
over `[-max_shift, max_shift]` without bounding the range by the trace length. The
CLI accepted any `usize`; a value near `i32::MAX` caused roughly 4.3 billion
correlation evaluations on a tiny trace.

**Impact.** Misalignment can corrupt TVLA/CPA evidence (false positives or
negatives), and a hostile or accidental `--max-shift` is a local denial of service.

**Fix.** `correlation` now computes the mean-centered, normalized Pearson
correlation over the overlap. `align` caps the search at
`samples_per_trace - 1`, because larger shifts have no overlap and can never win.
Tests cover amplitude/DC invariance, the one-sample overlap case, and an oversized
shift range.

---

### RT-06 — Medium — Synthetic generator allocated before dimension validation

**Affected code:** `crates/sicas-harness/src/synthetic.rs`.

**Observation.** `generate` and `generate_tvla` allocated trace vectors before
calling `TraceSet::from_traces`, which performs the dimension and total-sample
validation. A request such as a very large `--fixed-count` and `--samples` could
exhaust memory before the function returned `TooLarge`.

**Impact.** Local denial of service and possible OOM abort from a valid CLI request.

**Fix.** Both functions call `trace::validate_dimensions` before any allocation.
`validate_dimensions` is now `pub(crate)`. Regression tests cover oversized
single-group and TVLA generation.

---

### RT-07 — Medium — Trace I/O used one syscall per two-byte sample

**Affected code:** `crates/sicas-harness/src/trace.rs`,
`crates/sicas-harness/src/main.rs`.

**Observation.** `TraceSet::read_from` called `read_exact` for every `i16` sample,
and the CLI passed an unbuffered `File`. A 100M-sample trace (the format limit)
would issue roughly 100M read syscalls; the write path had the same problem. The
declared large-trace workflow was impractical, and a hostile file was a CPU/I/O
denial of service.

**Impact.** Availability and evidence-generation cost; not a memory-safety issue.

**Fix.** `read_from` wraps the reader in `BufReader`; `write_to` wraps the writer in
`BufWriter` and flushes explicitly. A counting-reader regression test verifies that
10,000 samples require fewer than 100 reads. The 100M-sample limit itself remains;
a streaming API for larger traces is a future hardening item.

---

### RT-08 — Medium — Unbounded leakage/timing configuration could exhaust memory or hang

**Affected code:** `crates/sicas-harness/src/leakage.rs`,
`crates/sicas-harness/src/timing.rs`, `crates/sicas-harness/src/main.rs`.

**Observation.** `LeakageConfig.samples` had no upper bound and the checker used
`Vec::with_capacity`, so a large `--samples` could abort on allocation. `dudect`
bounded `samples` but not `warmup`, so `--warmup` could run for an unbounded time.
The `ct` CLI bounded neither `trials` nor the total work.

**Impact.** Local denial of service and OOM abort from user-controlled options.

**Fix.** Added `leakage::MAX_SAMPLES` (1,000,000), `timing::MAX_TIMING_WARMUP`
(10,000,000), and `MAX_CT_TRIALS` (100), with `TooLarge` errors and regression
tests. The bounds are deliberately generous for evidence runs and small enough to
fail closed before memory exhaustion.

---

### RT-09 — Medium — Constant-time claim was broader than the assembly evidence

**Affected files:** `docs/SECURITY_CLAIMS.md`, `docs/CT_EVIDENCE.md`,
`tools/ct_asm_check.py`.

**Observation.** The 69-configuration assembly matrix inspects only
`ct::eq_u32`, `ct::select_u32`, and `ct::is_zero_u32`. The claim language
(“constant-time gadgets are data-independent”) and the proof-obligation table could
be read as covering `Masked::add_masked` and the masked ChaCha20 reference. They are
not in the matrix. `asm_masked_add` exists in the `ct_asm` example but is
intentionally excluded because it contains public loop and entropy-error branches.
RT-01 is exactly the kind of defect this gap allowed.

**Impact.** Claim/evidence mismatch and a blind spot in the constant-time
assurance argument.

**Fix (partial).** The claims and CT evidence now state the exact coverage: the
three core comparison/selection functions are assembly-checked; `lookup_u8` has a
public loop; `Masked::add_masked` is dudect/functional-tested; the bitsliced
reference is source-reviewed and differentially tested. RT-01 was fixed and has a
regression test. A dedicated reference-path assembly gate remains a hardening item
and is recorded in `docs/GAP_REPORT.md`.

**Residual risk.** A targeted gate now checks the `masked_word_planes` source
invariant and the `opt-level=0` x86_64 assembly (`tools/reference_ct_check.py`). It
does not cover every target or optimization level, so a future compiler or source
change outside that configuration could still reintroduce a secret-dependent branch.
The functional regression test alone checks bit extraction, not branchlessness.

---

### RT-10 — Medium — CI and release actions were not pinned to commit SHAs

**Affected files:** `.github/workflows/ci.yml`,
`.github/workflows/release.yml`.

**Observation.** Workflows used movable references such as
`actions/checkout@v4`, `dtolnay/rust-toolchain@master`, and
`model-checking/kani-github-action@v1`. The release workflow has `contents: write`,
`id-token: write`, and `attestations: write`. A compromised or retargeted action
could modify a release, mint an OIDC token, or exfiltrate repository credentials.
`PLAN.md` explicitly requires pinned actions.

**Impact.** Supply-chain compromise of CI or signed release artifacts.

**Fix.** Every third-party action is pinned to a 40-character commit SHA with a
version comment. `tools/check_workflow_pins.py` rejects unpinned references and is
run by `tools/local_proof.sh` and a dedicated CI job. `cargo-fuzz` and
`cargo-mutants` installs are version-pinned as well.

---

### RT-11 — Medium — Evidence tools could inspect stale artifacts

**Affected files:** `tools/ct_asm_check.py`, `tools/run-valgrind.sh`.

**Observation.** `ct_asm_check.py` selected `files[-1]` after lexicographic
sorting, and `run-valgrind.sh` selected `sort | tail -1`. Both target directories
can contain artifacts from earlier builds. The selected file could be stale,
producing evidence for code that is not under test.

**Impact.** False assembly or memory-safety evidence.

**Fix.** Both tools now select the newest artifact by modification time
(`st_mtime_ns` for Python, `find -printf '%T@'` for the shell script). A stronger
future fix is to clean the target directory or consume Cargo's JSON artifact
messages.

---

### RT-12 — Low — Original vision file contained superseded overclaims and a license mismatch

**Affected file:** repository-root `SiCaS` (now
`docs/archive/ORIGINAL_VISION.md`).

**Observation.** The original pitch claimed “universal” attenuation, “completely
flatten physical emissions,” “mathematically useless” snooping, “perfectly flat”
power, “0% overhead,” and MIT/Apache-2.0 licensing. Those claims are rejected in
`PLAN.md`; the project is GPL-3.0-only and makes bounded first-order claims. Leaving
the file at the repository root invited a reader to treat it as the current design.

**Impact.** Misleading public claims and license confusion; reputational and
audit-clarity risk.

**Fix.** The file was moved to `docs/archive/ORIGINAL_VISION.md` with a prominent
“superseded, not a security claim” header. The current claims remain in
`docs/SECURITY_CLAIMS.md`.

---

### RT-13 — Low — TVLA performance target exceeded the trace-format limit

**Affected files:** `PLAN.md` performance table, `docs/EVALUATION.md`,
`crates/sicas-harness/src/trace.rs`.

**Observation.** The plan listed “1M traces × 1k samples ≤ 60 s,” which is 1B
samples. The trace format rejects more than 100M total samples. The stated
performance target was therefore not loadable by the harness.

**Impact.** A hardware campaign sized to the plan would fail at parse time. This is
a requirements/evidence mismatch, not a memory-safety defect.

**Fix.** The plan now states “1M traces × 100 samples (100M total)” and records that
1M × 1k requires a streaming format and is out of scope for v1.0. The threat-model
trace-count budget is unchanged; only the per-trace sample count needed to fit the
format is explicit.

---

### RT-14 — High — Release tag name was interpolated into a shell script

**Affected file:** `.github/workflows/release.yml`.

**Observation.** The tag verification step used:

```yaml
run: |
  tag="${{ inputs.tag || github.ref_name }}"
  test "$(git rev-parse "refs/tags/${tag}^{commit}")" = "$(git rev-parse HEAD)"
```

Git ref names may contain `$` and parentheses, and `workflow_dispatch` input is
attacker-influenced. A tag such as `v1$(command)` is substituted into the script
before the shell parses it, so the command substitution can execute. The release
job has `contents: write`, `id-token: write`, and `attestations: write`.

**Impact.** A crafted tag name or dispatch input could execute commands in a
privileged release workflow. This is a CI script-injection and supply-chain defect.

**Fix.** The tag is passed through an environment variable, which the shell expands
as data rather than re-parsing:

```yaml
env:
  RELEASE_TAG: ${{ inputs.tag || github.ref_name }}
run: |
  tag="$RELEASE_TAG"
  test -n "$tag"
  test "$(git rev-parse "refs/tags/${tag}^{commit}")" = "$(git rev-parse HEAD)"
```

The `tag_name` and checkout `ref` expressions are action inputs, not shell code, and
remain expression-based.

---

### RT-15 — Low — Long-fuzz cache could reuse stale evidence

**Affected file:** `tools/run-fuzz-long.sh`.

**Observation.** The script reused an existing `$target.log` when it contained
`Done N runs`, without checking the commit or working-tree state. After a code
change, a cached PASS could be reported for a different tree.

**Impact.** Stale fuzz evidence; not a runtime vulnerability.

**Fix.** The script now refuses a dirty tracked tree, records the current commit in
`$target.commit`, and only reuses a log when the stamp matches. This makes the cache
commit-scoped.

---

### RT-16 — Low — CI fuzz job omitted the bitsliced batch target

**Affected files:** `.github/workflows/ci.yml`, `tools/run-fuzz-long.sh`.

**Observation.** The CI fuzz job ran `masked_ops`, `ct_lookup`, `trace_parser`, and
`chacha20`, but not `chacha20_batch`. The long-fuzz script also ran only four
targets. The local full proof runs the batch target, so a PR could break it without
CI or the long campaign catching the regression.

**Impact.** Coverage gap in the PR and long fuzz gates.

**Fix.** The CI fuzz job now runs `chacha20_batch` for 10,000 executions, and the
long-fuzz script includes `chacha20_batch` with the same 1,000,000-execution budget
as `chacha20`.

---

## Positive controls observed

- `#![forbid(unsafe_code)]` in the published logic crates; `sicas-zeroize` has one
  isolated, documented volatile-write boundary.
- Zero third-party runtime dependencies; `Cargo.lock` contains only workspace
  packages.
- `Secret<T>` has no equality, ordering, hashing, indexing, cloning, or copying;
  `Debug` is redacted.
- `Masked::ct_eq` compares the underlying values correctly even when the two
  representations use different masks (share-wise XORs cancel the masks).
- `ct::lookup_u8` scans the full table and selects by constant-time equality; the
  only branch is on the public loop counter.
- `Masked::and`, XOR, complement, rotate, shift, refresh, select, and addition
  match the reference; Kani covers the core gadgets.
- Entropy failure paths are `Result`-based and do not advance ChaCha20 state on
  failure.
- The bitsliced batch matches the independent unmasked ChaCha20 reference,
  including counter wraparound and scalar/batch interoperability.
- The trace parser rejects invalid magic, versions, zero dimensions, oversized
  inputs, and ragged traces; it is fuzzed.
- The release manifest validator checks artifact hashes, provenance, SBOM
  components/dependencies, workspace versions, timestamps, commit, and now the
  dirty flag.

## Claim impact

- **C2 (constant time):** narrowed to the exact assembly-covered functions. RT-01
  was a real gap in the reference path and is fixed; a reference assembly gate is
  still pending.
- **C3 (first-order masking):** the TVLA false negative (RT-02) is fixed; hardware
  TVLA remains pending. No hardware claim is made.
- **C4 (entropy fail-closed):** unchanged; no new defect found.
- **C5/C6 (zero deps, no unsafe):** unchanged.
- **C7 (reproducibility):** release dirty-tree binding improved; a fresh full proof
  and signed release remain pending.
- **C8 (model checking):** narrowed to core gadgets; the bitsliced composition is
  not Kani-proven.
- **C10 (signed release):** action pinning and tag verification improved; the
  production identity and Sigstore/SLSA run remain external gates.

## Retest commands

```bash
cargo test -p sicas-core -p sicas-reference -p sicas-harness --all-targets --all-features --locked
cargo test --workspace --doc --locked
python3 tools/check_stats.py
python3 tools/check_workflow_pins.py
python3 tools/test_check_workflow_pins.py
python3 tools/reference_ct_check.py --assembly
python3 tools/test_reference_ct_check.py
python3 tools/test_validate_release_manifest.py
sh -n tools/run-fuzz-long.sh
python3 tools/ct_asm_check.py --matrix --lto --output /tmp/sicas-asm.md
tools/run-valgrind.sh /tmp/sicas-valgrind
./tools/local_proof.sh quick
```

For the release-workflow fix, run a dry dispatch against a test tag and confirm the
`Verify the release tag points at the checked-out commit` step passes. Do not use a
production signing identity for a test.

## Verification

The final quick proof at `18b5123` passed every quick gate: fmt, workflow action-pin
checks and tests, strict Clippy, workspace tests, doctests, Kani, ASan, audit, deny,
the 69-configuration assembly matrix, zeroization assembly, leakage model, 204-case
statistics differential, Valgrind, release-validator tests, and the source-complete
evidence bundle. Full-workspace Miri also passed on the red-team code changes. The
full proof (mutation, coverage, five fuzz campaigns, cross-architecture tests, release
signing dry run, and reproducibility comparison) was not rerun for this review; those
rows remain attributed to their earlier commits. The summary is
[`evidence/local-proof-20261105-final/SUMMARY.md`](../evidence/local-proof-20261105-final/SUMMARY.md).

## Residual risk and external gates

- No independent red team or external audit has been performed.
- No hardware power/EM TVLA/CPA/DPA has been performed.
- No ARM/RISC-V runtime timing on physical targets.
- No production signing identity or SLSA attestation.
- `sicas-zeroize` remains experimental and unaudited.
- The reference bitsliced path is not in the automated assembly matrix.
- The 5× masked-ChaCha20 performance target remains open; the 10× hard ceiling is
  met.
- The evidence bundle is now source-complete and commit-checked, but it is not
  signed.

The project remains **SAL-2 locally** and must not be used to protect real secrets.
