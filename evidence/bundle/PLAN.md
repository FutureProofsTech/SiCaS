# SiCaS — Production Plan (Revised)

**Status:** Draft for approval  
**Date:** 2026-10-30  
**Hard constraints:** safe Rust, zero runtime dependencies, production security, no demo, strict formatting and Clippy.

---

## 0. Verdict

The SiCaS document is a strong research vision, but it is not yet a production plan. Three
items block it under the stated constraints:

1. **The LLVM pass cannot be built in safe Rust with zero dependencies.** A real LLVM pass
   requires C++/LLVM or unsafe FFI bindings (`llvm-sys`, `inkwell`). It also targets LLVM IR,
   not an "AST", and cannot safely mask arbitrary cryptographic code without understanding
   each algorithm.
2. **The eBPF/core-pair power-cancellation design is not technically or physically valid.**
   eBPF cannot call `sched_setaffinity`, schedule another thread, or synchronize two cores at
   instruction granularity. Two cores do not produce "negative power" that cancels the first
   core's leakage; power adds, it does not subtract.
3. **Jitter and harvested entropy are not primary side-channel defenses.** Random delays do
   not defeat DPA against an attacker who can align traces, and cache/thermal/network timing
   is not a secure TRNG. Inline assembly is unsafe, architecture-specific, and cannot be
   guaranteed to survive optimization.

**Recommended path:** ship a narrower, evidence-based v1.0 built on source-level typed
masking plus a leakage-assessment harness. Keep the compiler/runtime ideas as gated research
tracks, not production claims. If the original three pillars are non-negotiable, the project
must be labelled a research prototype, not production secure software.

---

## 1. Critical review of the current plan

| # | Issue in the current document | Why it is a problem | Required change |
|---|-------------------------------|---------------------|-----------------|
| 1 | Codename switches between **SiCaS** and **Project Mirage** | Confusing identity, docs, package names, and release artifacts | Use one name everywhere; remove all Mirage references |
| 2 | LLVM pass written in C++ | Violates "safe Rust, zero dependencies" | Drop from v1.0; replace with a safe Rust typed-masking API; keep a research spike only |
| 3 | "Rewrite the LLVM AST" | LLVM has IR, not an AST | Correct the terminology and the architecture |
| 4 | XOR-mask any "cryptographic load" | Boolean masking is not a universal transformation; AND, OR, addition, multiplication, and table lookups need specific masked gadgets | Restrict v1.0 to a defined gadget set and a reference algorithm; do not claim generic masking |
| 5 | "Compensation loop for M" | There is no generic compensation loop that makes arbitrary operations mask-correct | Implement and document each masked operation explicitly |
| 6 | S-box lookups on masked indices | A masked index still leaks through cache/memory access patterns; XOR does not fix this | Use constant-time full-table scan or a published masked S-box; measure cost |
| 7 | "Unmask only at the final memory write in an isolated register" | Registers are not physically isolated; the unmasking operation itself leaks | Define explicit declassification points; prefer masked output; never claim the write is leak-free |
| 8 | TRNG fetch on every sensitive load | Raw TRNG is slow, may fail or be biased, and is not a CSPRNG | Use an OS CSPRNG or an injected, audited entropy source; fail closed |
| 9 | eBPF calls `sched_setaffinity` and forces a core pair | eBPF is a sandboxed VM; it cannot make arbitrary syscalls or schedule threads | Remove from v1.0; runtime affinity is an optional measurement aid, not a security claim |
| 10 | Inverted dummy block cancels power on another core | Power is additive; separate cores have separate domains, clocks, and routing; exact cancellation is impossible | Drop the "flat power rail" claim entirely |
| 11 | Random jitter defeats DPA | DPA can align traces or average through noise; jitter also adds timing leakage and DoS risk | Make jitter optional, bounded, off by default, and never a primary claim |
| 12 | "Architecture-independent inline assembly" | `asm!` is unsafe and architecture-specific; compilers may remove or transform dummy work | No inline asm in v1.0; use safe `black_box` barriers only where verified |
| 13 | Entropy from cache misses, thermal sensors, network timing | Low entropy, attacker-influenceable, platform-specific, not a TRNG | Use OS CSPRNG or host-provided entropy; no weak fallback |
| 14 | "ARM (Android/iOS) + eBPF", "RISC-V rmask" | iOS does not allow eBPF or arbitrary kernel hooks; Android restricts eBPF; `rmask` is not a universal RISC-V primitive | Publish an honest support matrix; stage platforms |
| 15 | "Zero overhead", "0% overhead", "completely flat", "mathematically useless" | Impossible to guarantee and damages credibility | Replace with measured budgets and bounded, quantified claims |
| 16 | No threat model, assets, adversary, or non-goals | "Production secure" is undefined without them | Write `THREAT_MODEL.md` and `SECURITY_CLAIMS.md` before code |
| 17 | No constant-time or leakage verification | A masking design without measurement is a hypothesis, not evidence | Add TVLA, CPA, dudect-style timing tests, and assembly inspection |
| 18 | ELMO simulation treated as physical validation | ELMO targets ARM Cortex-M and cannot validate x86_64 or real hardware effects | Simulation is pre-silicon only; production claims need real hardware traces |
| 19 | Six-week single-developer plan | Compiler + kernel + cross-architecture + audit is a multi-month/year effort | Use phase gates and a realistic 8–12 month SAL-4 v1.0 schedule |
| 20 | No audit, CVE process, reproducible builds, or maintenance plan | These are required for production secure software | Add external audit, signed releases, SBOM, and incident response |
| 21 | Marketing to Signal/hardware-wallet communities before evidence | Reputational and ethical risk; overclaiming in security is harmful | Publish limitations and evidence first; engage reviewers early |
| 22 | Dual-use side-channel work | Defensive, but sensitive | Add a responsible-use policy and disclosure process |

### What is worth keeping

- The goal of reducing side-channel leakage is valid.
- Boolean masking with fresh randomness is a real, well-studied technique.
- The concern that compiler optimization can break masking is valid.
- The concern that OS scheduling and core placement affect measurements is valid.
- Trace alignment is a real problem, even if jitter is not a complete answer.
- Cross-architecture ambition is good, but it must be staged and evidenced.

---

## 2. Recommended v1.0 scope

### In scope

- **`sicas-core`**: safe, `no_std`, zero-dependency types and gadgets for constant-time
  secret handling and first-order Boolean masking.
- **`sicas-harness`**: safe, `std`, zero-dependency TVLA/CPA analysis and reporting.
- **`sicas-cli`**: safe, `std`, zero-dependency command-line orchestration.
- **Reference integration**: masked ChaCha20 (ARX-friendly), validated against RFC 8439
  test vectors and measured on real hardware.
- **Platforms**: Linux x86_64 for tooling; ARM Cortex-M4 (or equivalent) for hardware
  leakage measurement; cross-compilation checks for AArch64 and RISC-V.
- **Evidence**: threat model, claims matrix, TVLA/CPA/DPA/SPA/template/higher-order reports,
  raw traces, reproducible analysis, independent red team, external audit, and the full SAL-4
  evidence bundle before a 1.0 production claim.

### Out of scope for v1.0

- LLVM pass, eBPF, kernel modules, iOS/Android deployment.
- Power-rail cancellation, "zero leakage", "unbreakable", "universal".
- Higher-order masking **as a claimed protection**; invasive attacks; microarchitectural
  attacks. Fault-injection tests are used to verify fail-closed behavior, but v1.0 does not
  claim resistance to a capable glitcher.
- Shipping new production cryptographic primitives.
- Windows/macOS runtime adapters (planned for v1.1).

### Non-negotiable engineering rules

- No feature is "done" without a test, a threat-model entry, a claim/evidence link, and a CI gate.
- No security claim is made from a demo, a simulation, or a single successful run.
- Every public API documents its leakage contract and its assumptions.
- Every unsafe block, if any, is isolated, justified with `// SAFETY:`, and reviewed.
- Every release is reproducible and signed.
- The v1.0 target is **SAL-4** (Section 8.1).
- Independent red-team review is mandatory before v1.0; critical/high findings block release.
- Every verification tool is qualified; its limitations and uncovered paths are documented.

### Code quality and proof standard

The project standard is: every line is reviewable, every public item is documented, every
security property has a proof obligation, and every proof obligation has a tool and a
published result. No code is accepted because it "looks right."

**Definition of done for every module and function**

- Public API docs with a safety/leakage contract and tested examples.
- Unit tests for nominal, boundary, invalid-input, and error paths.
- Property/metamorphic tests for algebraic invariants.
- Compile-fail tests for forbidden API use.
- A fuzz target for every parser or byte-oriented input.
- Miri and the applicable sanitizers.
- Kani/model checking for critical invariants where tractable.
- Assembly inspection for constant-time and other security-critical code.
- Mutation testing coverage and a reviewed coverage report.
- Two-person review; security-critical changes require independent review.
- No `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, or `dbg!` in production paths.
- No secret-dependent branch or index; no secret in logs, errors, `Debug`, or panic messages.
- No silent fallback; security failures must fail closed.

**Proof obligations**

| Property | Proof obligation | Primary tools | Evidence |
|----------|------------------|---------------|----------|
| Memory safety / no UB | No out-of-bounds, use-after-free, aliasing violation, or uninitialized read | Miri, ASan, UBSan, Valgrind | Reports per target and commit |
| Panic-freedom and bounds | No panic, overflow, or slice-index failure on untrusted input | Kani, fuzzing, restriction lints, `-C overflow-checks` | Proofs, fuzz stats, test reports |
| Functional correctness | Matches RFC vectors and independent references | Unit/integration tests, differential tests, Kani | Test logs and vectors |
| Constant time | Execution is independent of secret values | dudect, ctgrind, assembly inspection | Timing reports and disassembly |
| Masking correctness | Shares combine correctly; no first-order leakage in the model | Property tests, Kani, TVLA/CPA | Proofs and raw traces |
| Entropy fail-closed | Entropy failure cannot silently produce predictable masks | Fault injection, unit tests, code review | Failure-injection report |
| Parser robustness | Malformed or hostile input cannot crash or hang the tool | Fuzzing, Miri, sanitizers, timeouts | Fuzz corpus and crash-free campaign |
| API misuse resistance | Forbidden secret operations do not compile | Compile-fail tests, independent review | Compile-fail suite and review record |
| Supply-chain integrity | No hidden runtime dependencies; artifacts are reproducible | `cargo-audit`, `cargo-deny`, `cargo-vet`, SBOM, reproducible builds | Dependency and build reports |
| Performance | Overhead stays within the declared budget | `cargo-bloat`, `iai-callgrind`, `perf`, Callgrind | Benchmark reports |
| Documentation | Every public item is documented and links resolve | `rustdoc -D warnings`, `missing_docs` | Doc build log |
| Release integrity | Artifacts are signed, traceable, and reproducible | Sigstore/minisign, SLSA provenance, SBOM | Signature and provenance verification |

**No proof, no claim.** Every entry in `SECURITY_CLAIMS.md` links to evidence. If the evidence
is missing, the claim is downgraded or removed. "Use every possible tool" means exhaustive
property coverage, not running tools for appearance.

---

## 3. Revised architecture

### 3.1 `sicas-core` — typed secrets and masking

- `#![forbid(unsafe_code)]`, `#![no_std]`, zero dependencies.
- `Secret<T>`:
  - redacted `Debug`;
  - no `PartialEq`, `Ord`, `Hash`, `Index`, `Deref`, or implicit conversion to `T`;
  - explicit `declassify()` boundary with documentation.
- `Choice` / `ct_eq` / `ct_select` / `ct_swap` / `ct_is_zero` / `ct_lookup`.
- `Masked<T>` with two shares (`share0 ^ share1 = value`):
  - `mask`, explicit `unmask`, linear operations, rotate/shift;
  - AND with a fresh mask;
  - no secret-dependent branches or indexing.
- `EntropySource` trait:
  - host-provided or OS adapter;
  - deterministic test source for tests only;
  - fail-closed error type.
- `SecretBytes` / `MaskedBytes` with a documented zeroization policy.
- Leakage contracts per operation:
  - assumptions (freshness, independence, first-order model);
  - guarantees (what is masked, what is declassified, what is not protected).

**Important:** software masking can be undone by compiler optimization. v1.0 must include
assembly inspection for every supported compiler/target and `black_box` barriers at share
boundaries. If optimization still breaks the invariants, that is a research finding, not a
reason to silently claim success.

### 3.2 `sicas-harness` — evidence, not marketing

- Vendor-neutral trace format with metadata: target, firmware hash, clock, samples, trigger,
  algorithm, key, plaintext, mask, acquisition parameters.
- Preprocessing: trim, align, filter.
- TVLA fixed-vs-random with Welch's t-test; report max `|t|`, exceedance counts, and
  confidence intervals. Use `|t| > 4.5` as the standard leakage indicator.
- CPA with Hamming-weight/distance models and correct-key rank.
- Synthetic trace generator with known SNR to validate the harness itself.
- Deterministic JSON/Markdown reports including tool version, commit, and hashes.
- Hardware capture stays external (ChipWhisperer/Python or vendor tooling); the Rust harness
  analyzes exported traces. This keeps the core vendor-neutral and testable.

### 3.3 `sicas-runtime` (optional, v1.1)

- OS entropy adapter, secure memory adapter, optional thread affinity for measurement
  reproducibility.
- No claim that affinity or jitter cancels power leakage.
- If unsafe is required for syscalls or volatile zeroization, it lives only here, behind a
  documented boundary.

### 3.4 Research tracks (gated, not v1.0 claims)

- **Compiler track:** investigate a MIR/LLVM transformation only if Phase 1–3 evidence shows
  source-level barriers are insufficient. A `rustc_private` MIR pass is nightly-only and
  high-maintenance; a C++ LLVM pass violates the constraints. Gate with an ADR and a
  semantics-preservation plan. A safer first use of LLVM IR/MIR is **verification** (detecting
  secret-dependent branches or indices), not transformation.
- **Runtime track:** investigate whether OS scheduling isolation measurably improves SNR.
  Do not claim power cancellation. eBPF may be evaluated later for observability/telemetry
  (detecting target functions, collecting timing), never for scheduling or power cancellation.
- **Jitter track:** measure whether bounded delays improve resistance to a defined attacker.
  If the effect is not statistically significant, do not ship it.

---

## 4. Threat model and claims

### 4.1 Assets

- Long-term and session cryptographic keys.
- Plaintext and intermediate secret values.
- Mask shares and RNG state.
- Control flow and memory-access patterns derived from secrets.

### 4.2 Adversary (v1.0)

- Passive physical attacker with power/EM probe, oscilloscope, known or selected plaintext,
  and a trigger signal.
- Can collect a bounded number of traces from the same device under normal operating
  conditions. The v1.0 declared budget is up to 1,000,000 traces for first-order TVLA/DPA and
  up to 100,000 traces for CPA; exact per-target numbers are published with the results.
- No invasive probing, no fault injection, no firmware modification, no malicious OS or
  toolchain. Red-team engagements may exceed this model; findings outside it are reported as
  hardening evidence, not as violations of the claimed threat model.

### 4.3 Out of scope as security claims

- Higher-order DPA, profiling template attacks, fault-injection resistance, invasive attacks,
  cache-timing and microarchitectural attacks, supply-chain attacks, side channels in I/O or
  the OS, and malicious hardware.

Red-team testing still attempts these attacks where feasible. A passed test is evidence for
that engagement, not a guarantee against all future attacks.

### 4.4 Claims matrix (initial)

| ID | Claim | Mechanism | Evidence required | Status |
|----|-------|-----------|-------------------|--------|
| C1 | Safe API prevents accidental secret-dependent branching/indexing | `Secret<T>` type design; no `Eq`/`Ord`/`Index` | Compile-fail tests, API review | Planned |
| C2 | Constant-time gadgets are data-independent on supported targets | Bitwise formulas, no branches | Assembly inspection, dudect-style tests | Planned |
| C3 | First-order Boolean masking reduces first-order DPA below TVLA threshold for the reference algorithm | Two-share masking, fresh randomness | TVLA report, raw traces, target details | Planned |
| C4 | Entropy failure is fail-closed | `Result`-based entropy API, no weak fallback | Fault-injection tests, code review | Planned |
| C5 | Zero runtime dependencies | Workspace policy, CI dependency check | `cargo metadata`/`cargo tree` report | Planned |
| C6 | No unsafe code in core logic crates | `#![forbid(unsafe_code)]` | CI, source audit | Planned |
| C7 | Builds are reproducible | Pinned toolchain, locked deps, deterministic flags | Two-build hash comparison | Planned |
| C8 | Critical invariants are model-checked | Kani/CBMC, Creusot/Prusti/Verus where applicable | Proof artifacts, assumptions, uncovered paths | Planned |
| C9 | v1.0 meets SAL-4 | Layered verification, red team, audit, evidence bundle | Independent verification of every SAL-4 criterion | Planned |
| C10 | Release artifacts are signed and traceable | Sigstore/minisign, SLSA provenance, SBOM | Signature and provenance verification report | Planned |

**Non-claims:** no protection against higher-order DPA, fault injection, invasive attacks,
all algorithms, all platforms, all compilers, or all side channels. No "zero leakage" claim.
The public claim is "SAL-4 under threat model T".

---

## 5. Repository layout

```text
SiCaS/
├── Cargo.toml                 # workspace, resolver, shared lints
├── Cargo.lock                 # committed (workspace ships binaries)
├── rust-toolchain.toml        # pinned toolchain
├── .cargo/config.toml         # aliases, target settings
├── .github/workflows/ci.yml
├── README.md
├── LICENSE                    # GPL-3.0-only
├── SECURITY.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── CHANGELOG.md
├── docs/
│   ├── THREAT_MODEL.md
│   ├── SECURITY_CLAIMS.md
│   ├── ARCHITECTURE.md
│   ├── SUPPORT_MATRIX.md
│   ├── ASSURANCE.md
│   ├── CODE_QUALITY.md
│   ├── PROOF_OBLIGATIONS.md
│   ├── TOOLING.md
│   ├── RED_TEAM.md
│   ├── EVALUATION.md
│   ├── EVIDENCE.md
│   ├── RELEASE.md
│   └── adr/
├── crates/
│   ├── sicas-core/            # no_std, forbid unsafe, zero deps
│   ├── sicas-harness/         # std, zero deps
│   ├── sicas-cli/             # std, zero deps
│   ├── sicas-runtime/         # optional, std
│   └── sicas-platform/        # optional, isolated unsafe boundary
├── tests/
│   ├── compile-fail/
│   ├── integration/
│   └── leakage/
├── red-team/                  # scoped PoCs, traces, scripts, sanitized
├── evidence/                  # generated SAL-4 evidence bundle
├── xtask/                     # zero-dep task runner
└── fuzz/                      # optional, non-published
```

---

## 6. Toolchain, formatting, and lint policy

### 6.1 Pin the toolchain

`+nightly` alone is not reproducible. Pin an exact nightly for formatting and Clippy, and a
stable MSRV for builds/tests.

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.99.0"
components = ["rustfmt", "clippy", "llvm-tools-preview"]
profile = "minimal"
```

CI uses an exact pinned nightly for the strict lint job:

```bash
rustup toolchain install nightly-2026-08-03 \
  --component rustfmt,clippy,miri,rust-src,llvm-tools-preview
```

### 6.2 Required commands

```bash
# Format (pinned; do not rely on a floating "nightly")
cargo +nightly-2026-08-03 fmt --all -- --check

# Strict Clippy
cargo +nightly-2026-08-03 clippy \
  --workspace --all-targets --all-features --locked \
  -- -D warnings -D clippy::pedantic -D clippy::nursery

# Tests
cargo test --workspace --all-targets --all-features --locked
cargo test --workspace --doc --locked

# Documentation
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --locked

# MSRV build
cargo +1.99.0 check --workspace --all-targets --all-features --locked
```

### 6.3 Lint policy

- The requested pedantic + nursery policy is adopted.
- Do **not** blanket-allow lints. Any allow must be local, with a reason and a tracking issue.
- Nursery lints are unstable; pin the nightly and review new lints on a schedule.
- `clippy.toml` should set `msrv`, `disallowed-methods`, and `disallowed-types` for dangerous
  APIs (`mem::transmute`, raw pointer casts, `unsafe` escape hatches, variable-time compares).
- Security-critical crates also deny the restriction lints `unwrap_used`, `expect_used`,
  `panic`, `todo`, `unimplemented`, `dbg_macro`, `indexing_slicing`,
  `arithmetic_side_effects`, `integer_division`, and `as_conversions`. Use explicit `Result`
  handling and checked conversions instead.
- Rust lints: `missing_docs`, `missing_debug_implementations`, `unreachable_pub`,
  `unused_must_use`, and `unsafe_op_in_unsafe_fn`.
- Rustdoc lints: `broken_intra_doc_links`, `missing_crate_level_docs`, and
  `private_intra_doc_links`.
- Tests and examples are held to the same lint standard (`--all-targets`). Test-only
  `unwrap`/`expect` may be allowed locally with a reason.
- `#![forbid(unsafe_code)]` in `sicas-core`, `sicas-harness`, and `sicas-cli`.
- `sicas-platform` may use `#[allow(unsafe_code)]` only in small, documented modules; it must
  pass Miri where supported and receive explicit review.

### 6.4 Zero-dependency policy

- No third-party crates in `[dependencies]` or `[build-dependencies]` of published crates.
- `[dev-dependencies]` empty for published crates. Verification tools that require crates
  live in a non-published `fuzz/` or `xtask/` member and are optional.
- `std` is allowed in `std` crates; `core`/`alloc` only in `no_std` crates.
- CI fails if `cargo metadata` shows a non-workspace normal dependency.
- No vendored C/C++/assembly in core logic.
- Zero dependencies must not become "roll your own crypto": v1.0 ships masking/verification
  infrastructure, not new production primitives. The reference ChaCha20 is for validation.

### 6.5 CI jobs

1. `fmt` — pinned nightly, `--check`.
2. `clippy` — pinned nightly, all targets/features, strict groups.
3. `test` — stable + MSRV, unit/integration/doc tests, `cargo-nextest`.
4. `miri` — core and harness logic (not OS adapters).
5. `sanitizers` — nightly ASan/UBSan/TSan/MSan where supported.
6. `model-checking` — Kani smoke on critical gadgets (full suite nightly).
7. `fuzz` — 60 s smoke on PRs; ≥1 h nightly; release-blocking campaign.
8. `mutation` — `cargo-mutants` on core logic.
9. `coverage` — `cargo-llvm-cov`; thresholds and trend report.
10. `docs` — `-D warnings`.
11. `cross` — `thumbv7em-none-eabihf`, `riscv32imac-unknown-none-elf`,
    `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-gnu`.
12. `no-deps` — dependency policy check.
13. `supply-chain` — `cargo-audit`, `cargo-deny`, `cargo-vet`, SBOM, secret scanning.
14. `reproducible` — build twice, compare artifact hashes.
15. `performance` — `cargo-bloat`, `iai-callgrind`, regression thresholds.
16. `leakage` — synthetic tests on every PR; hardware-in-the-loop weekly/manual.
17. `red-team-regression` — replay the red-team attack catalog before each release.

---

## 7. Phase plan

### Phase 0 — Scope lock and safety case (week 1)

**Tasks**
- P0.1 Fix naming: SiCaS only.
- P0.2 Write `docs/THREAT_MODEL.md` (assets, adversary, assumptions, out-of-scope).
- P0.3 Write `docs/SECURITY_CLAIMS.md` (claim IDs, evidence, non-claims).
- P0.4 Write `docs/SUPPORT_MATRIX.md` (v1.0: Linux x86_64 + Cortex-M4; later: AArch64,
  RISC-V; not supported: iOS/Android/eBPF/kernel).
- P0.5 ADRs: safe Rust/unsafe boundary; zero dependencies; drop eBPF power balancing; drop
  LLVM pass from v1.0; entropy fail-closed; first-order masking scope; toolchain pin.
- P0.6 Repository skeleton, licenses, `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`.
- P0.7 CI skeleton with fmt, clippy, test, docs, Miri, sanitizers, Kani smoke, and fuzz smoke.
- P0.8 Write `docs/ASSURANCE.md`: SAL-4 criteria, tool qualification matrix, test cadence,
  evidence-bundle format, and claim-review process.
- P0.9 Write the red-team scope, rules of engagement, authorization template, attack catalog,
  severity/SLA, and disclosure policy.
- P0.10 Write the formal-verification plan: critical invariants, tool choices, assumptions,
  and uncovered paths.
- P0.11 Write `docs/CODE_QUALITY.md` and `docs/PROOF_OBLIGATIONS.md`: definition of done,
  lint policy, restriction lints, proof-obligation table, and evidence links.
- P0.12 Gate review: approve revised scope, SAL-4 target, code-quality standard, and red-team
  plan or stop.

**Deliverables:** approved charter, threat model, claims matrix, assurance plan, code-quality
standard, proof obligations, red-team plan, ADRs, green CI skeleton.

**Acceptance:** every claim has an evidence plan; no absolute "unbreakable/zero-leakage"
language; no unresolved architecture contradiction; SAL-4, code-quality, and red-team criteria
approved.

---

### Phase 1 — `sicas-core` (weeks 2–5)

**Tasks**
- P1.1 Implement `Choice`, `ct_eq`, `ct_select`, `ct_swap`, `ct_is_zero`, `ct_lookup`.
- P1.2 Implement `Secret<T>` with redacted `Debug` and no leaky trait impls.
- P1.3 Implement `Masked<T>` for `u8`, `u32`, `u64`: mask/unmask, XOR, NOT, AND with fresh
  mask, rotate, shift.
- P1.4 Define `EntropySource` and a deterministic test source.
- P1.5 Define `SecretBytes` / `MaskedBytes` and the zeroization policy.
- P1.6 Unit tests, property tests with a custom PRNG, differential tests, compile-fail tests.
- P1.7 Inspect assembly for constant-time gadgets on x86_64 and Cortex-M4; document compiler,
  flags, and observed instructions.
- P1.8 Add fuzz targets for masked operations, entropy handling, and any parser/serialization.
- P1.9 Model-check critical invariants with Kani: panic-freedom, bounds, overflow, and mask
  correctness for bounded inputs.
- P1.10 Run mutation testing (`cargo-mutants`) and require a ≥90% mutation score on core
  security logic.
- P1.11 Verify the compiler matrix: opt-level 0/1/2/3/s, LTO thin/fat, codegen-units 1/16,
  panic=abort/unwind; inspect assembly for each configuration.
- P1.12 Run Miri, ASan, UBSan, Valgrind, and strict Clippy on every PR; publish the reports.

**Deliverables:** published `sicas-core` API, tests, assembly reports, leakage contracts,
Kani proofs, fuzz corpus, mutation/coverage reports.

**Acceptance:** `#![forbid(unsafe_code)]`; zero deps; all tests pass; Miri/ASan/UBSan/Valgrind
clean; Kani proofs pass for the declared bounds; mutation score ≥90%; fuzz targets run 24 h
without crashes before release; compiler-matrix assembly reports show no secret-dependent
branches or indices; every public item documented.

---

### Phase 2 — `sicas-harness` (weeks 6–9)

**Tasks**
- P2.1 Specify the trace format and implement reader/writer.
- P2.2 Implement trim, alignment, and filtering.
- P2.3 Implement fixed-vs-random TVLA with Welch's t-test and reporting.
- P2.4 Implement CPA (Hamming weight/distance) and correct-key rank.
- P2.5 Implement a synthetic trace generator with configurable leakage/SNR.
- P2.6 Implement `sicas-harness analyze` CLI.
- P2.7 Emit deterministic JSON/Markdown reports with metadata and hashes.
- P2.8 Validate: detect a known leak at the expected trace count; do not false-positive on
  noise-only traces; pass a clean/masked target.
- P2.9 Structured-fuzz the trace parser, CLI, and configuration with libFuzzer/AFL++/honggfuzz;
  keep a regression corpus.
- P2.10 Differential-test statistics against an independent reference implementation
  (Python/NumPy scripts are external test tooling, not runtime dependencies).
- P2.11 Adversarially review the statistical methodology with known-leaky and clean controls.

**Deliverables:** harness binary, trace spec, validation report, synthetic benchmarks, fuzz
corpus, differential-test report.

**Acceptance:** synthetic leak detection and false-positive targets pass; differential results
match the reference; 24 h continuous fuzzing without crashes; reports reproducible; zero deps;
strict Clippy clean.

---

### Phase 3 — Reference integration and physical evidence (weeks 10–15)

**Tasks**
- P3.1 Implement masked ChaCha20 using `sicas-core`; validate against RFC 8439 vectors.
- P3.2 Constant-time review: assembly inspection and dudect-style timing tests.
- P3.3 Capture power/EM traces on real hardware (ChipWhisperer or equivalent).
- P3.4 Run TVLA fixed-vs-random; publish max `|t|`, trace count, and acquisition parameters.
- P3.5 Benchmark cycles/byte and overhead vs. an unmasked reference.
- P3.6 If AES is added, use a constant-time full-table scan or a published masked S-box; do
  not use masked indices directly.
- P3.7 Run the full side-channel attack suite: SPA, DPA, CPA, template attacks, and
  higher-order analysis on both power and EM traces.
- P3.8 Test multiple devices, temperature/voltage variation, and multiple trace counts; report
  confidence intervals and failure boundaries.
- P3.9 Perform fault-injection tests (voltage/clock glitching) and RNG stuck-at/bias tests
  where lab hardware allows.
- P3.10 Run leakage tests across the compiler matrix (opt levels, LTO) to prove masking
  survives optimization.
- P3.11 Commission an independent red-team review of the reference integration.

**Deliverables:** reference integration, test vectors, raw traces, TVLA/CPA/DPA report,
fault-injection report, benchmark, red-team report.

**Acceptance:** test vectors pass; TVLA `|t| < 4.5` at the declared trace count; no key
recovery within the declared attack budget; all critical/high red-team findings resolved;
overhead within the agreed budget; raw data and scripts published for independent re-analysis.

**Hardware note:** simulation alone is pre-silicon evidence. Production claims require real
hardware. If the budget is truly zero, borrow/rent a ChipWhisperer or partner with a lab; do
not claim production validation from ELMO alone.

---

### Phase 4 — Analyzer research (weeks 16–19, go/no-go)

**Tasks**
- P4.1 Evaluate options: type-level enforcement (already in Phase 1), source scanner, MIR pass
  via `rustc_private`, external analysis tool.
- P4.2 Prototype a scanner for banned patterns (`unsafe`, `asm!`, secret `==`, secret indexing,
  secret `if`/`match`, secret `Debug`, variable-time compares).
- P4.3 Measure false positives/negatives on a seeded benchmark corpus.
- P4.4 Write an ADR: productionize, defer, or reject.
- P4.5 Red-team the analyzer with macros, generated code, `unsafe`, FFI, trait tricks, and
  aliasing; measure bypasses and false positives/negatives.

**Acceptance:** evidence-based decision; bypass report and detection rates documented. If
deferred, no production claim depends on it.

---

### Phase 5 — Runtime adapters (weeks 20–22, optional)

**Tasks**
- P5.1 OS entropy adapter with fail-closed behavior.
- P5.2 Secure memory/zeroization adapter, with unsafe isolated and documented.
- P5.3 Optional thread affinity for measurement reproducibility (not a security claim).
- P5.4 Optional jitter experiment; measure effect; keep off by default.
- P5.5 Fault-injection tests: entropy failure, RNG stuck-at/bias, truncated input, and
  concurrency stress.
- P5.6 Red-team the runtime adapters and their failure paths.

**Acceptance:** entropy failure tests pass; fail-closed behavior verified; unsafe boundary
reviewed; red-team findings resolved; no overclaims; platform tests pass.

---

### Phase 6 — Productionization, red team, and audit (weeks 23–30+)

**Tasks**
- P6.1 Independent external security audit (masking, Rust unsafe boundary, supply chain, API
  misuse, cryptographic assumptions).
- P6.2 Full external red-team engagement against the attack catalog in Section 9.3.
- P6.3 Formal/model-checking sign-off for all critical invariants; document assumptions and
  uncovered paths.
- P6.4 Fix all findings; re-run the complete SAL-4 evidence suite; retest.
- P6.5 Reproducible builds, signed releases, SBOM, checksums, SLSA provenance.
- P6.6 User guide, integration guide, security claims, limitations, migration guide.
- P6.7 Performance and compatibility matrix.
- P6.8 `SECURITY.md`, CVE/incident-response process, disclosure policy, bug-bounty plan.
- P6.9 Publish the SAL-4 evidence bundle and release candidate for public review.
- P6.10 v1.0 release only after independent verification of every SAL-4 criterion.

**Acceptance:** SAL-4 criteria met; no open critical/high audit or red-team findings; all gates
green; claims backed by reproducible evidence; release checklist complete.

---

### Phase 7 — Maintenance and continuous assurance (ongoing)

- Quarterly toolchain/lint review and dependency-policy check.
- Quarterly internal red-team regression; annual external red team and audit.
- Continuous fuzzing and mutation-testing campaigns.
- Hardware regression runs for supported targets.
- CVE response SLA, security advisories, and bug-bounty operations.
- Deprecation policy and MSRV policy.
- Re-certify SAL-4 for every major release; downgrade the claim if evidence is missing.

---

## 8. Verification and assurance program

### 8.1 Assurance target: SAL-4

SiCaS uses a measurable assurance target: **SiCaS Assurance Level 4 (SAL-4)**.

| Level | Requirements | Allowed public claim |
|-------|--------------|----------------------|
| SAL-0 | Demo or prototype only | "experimental" |
| SAL-1 | Unit/integration tests + CI | "tested" |
| SAL-2 | SAL-1 + Miri, sanitizers, fuzzing, static analysis | "hardened" |
| SAL-3 | SAL-2 + formal/model checking of critical invariants + real-hardware TVLA + external audit | "audited" |
| SAL-4 | SAL-3 + independent red team + reproducible/signed builds + no open critical/high findings + public evidence bundle | "SAL-4 under threat model T" |

SAL-4 is the v1.0 target. A release may claim SAL-4 only when every criterion is met and
independently verified. Otherwise the release is labelled with its actual level. Security
documentation must never use "unbreakable", "zero leakage", or similar unverifiable language.

**Schedule impact:** full SAL-4 adds roughly 6–10 weeks for red team, audit, remediation, and
retesting. A realistic single-developer v1.0 is therefore **8–12 months**, not 6–9.

### 8.2 Hardcore testing and tool matrix

Every tool is qualified for a specific property. No tool is treated as a silver bullet.

| Area | Tools / methods | What it proves | Gate |
|------|-----------------|----------------|------|
| Formatting and lints | `rustfmt` nightly, Clippy pedantic + nursery, rustc lints, `cargo-hack`, `cargo-udeps`, `cargo-machete`, `cargo-expand`, `cargo-public-api`, `cargo-semver-checks` | Style, common bugs, feature combinations, macro expansion, API surface | Every PR / release |
| Memory safety / UB | Miri, ASan, UBSan, Valgrind Memcheck/Massif/Helgrind/DRD, `-C overflow-checks`, `-C debug-assertions` | No UB, OOB, use-after-free, leaks, overflow in tested paths | Every PR / nightly |
| Concurrency | TSan, stress tests, `rr` record/replay, optional `loom` in a non-published test crate | Races, deadlocks, ordering bugs | Nightly |
| Uninitialized memory | MSan where supported, Valgrind | Uninitialized reads | Nightly |
| Functional correctness | Unit tests, integration tests, RFC vectors, golden files, differential tests, doc tests, `cargo-nextest`, snapshot tests | Correctness against references | Every PR |
| Property / metamorphic | Custom zero-dependency property harness; optional `proptest` in a non-published test crate | Invariants, algebraic laws, share independence | Every PR / nightly |
| Fuzzing | `cargo-fuzz`/libFuzzer, AFL++, honggfuzz, structured fuzzing, corpus regression, coverage-guided fuzzing | Crashes, parser bugs, panics, DoS | Nightly / release |
| Model checking | Kani/CBMC for bounds, overflow, panic-freedom, critical invariants; Creusot/Prusti/Verus where applicable | Exhaustive proofs for bounded models | Nightly / release |
| Mutation testing | `cargo-mutants` | Test-suite strength and blind spots | Nightly / release |
| Coverage | `cargo-llvm-cov`, `grcov`, `llvm-profdata` | Untested code paths (not proof of security) | Nightly |
| Constant time | dudect, ctgrind/Valgrind, assembly inspection (`cargo asm`, `objdump`), timing stress tests | Data-independent execution for reviewed functions | Every release |
| Compiler matrix | Stable + pinned nightly; opt-level 0/1/2/3/s; LTO thin/fat; codegen-units 1/16; panic=abort/unwind; debug/release; target-feature variants | Optimization and LTO do not break masking or constant-time properties | Every release |
| Platform matrix | x86_64, aarch64, thumbv7em, riscv32/riscv64; QEMU + real hardware | Portability and target-specific correctness | Every PR / release |
| Side-channel | TVLA fixed-vs-random, SPA, DPA, CPA, template attacks, higher-order tests, power and EM, multiple devices, temperature/voltage variation | Leakage under the defined threat model | Every release + hardware run |
| Fault injection | Voltage/clock glitching, RNG stuck-at/bias, entropy failure, truncated/malformed traces, panic/unwind paths | Fail-closed behavior and robustness | Release, where hardware allows |
| Performance / resources | `cargo-bloat`, `iai-callgrind`, `perf`, Callgrind, heaptrack/Massif, `hyperfine` | Overhead, memory use, binary size, regressions | Every release |
| Runtime observation | `strace`, `perf`, `rr`, `bpftrace` (observability only) | Syscall behavior, scheduling noise, no hidden I/O | Nightly |
| Supply chain | `cargo-audit`, `cargo-deny`, `cargo-vet`, `cargo-semver-checks`, `cargo-about`, `cargo-cyclonedx`, `cargo-auditable`, SBOM, reproducible builds, Sigstore/minisign, SLSA provenance, secret scanning | Build and release integrity | Every release |
| API misuse | Compile-fail tests, custom `rustc` harness (or `trybuild` in a non-published test crate), misuse fuzzing, `cargo-public-api` | Forbidden operations fail to compile; safe API is hard to misuse | Every PR |
| Independent review | Two-person review for security-critical changes; external design/threat-model review | Design flaws, unsafe assumptions, API misuse that tools cannot see | Every release / critical change |
| Red team | Independent adversarial testing and attack-chain analysis | Realistic bypasses, composition failures, operational weaknesses | Pre-1.0 and annual |

**Tool qualification rule:** every tool must have a documented property, scope, version, and
known limitations. The inventory is reviewed quarterly; a new tool is adopted when it covers
a property that is not already covered. The goal is exhaustive property coverage, not
exhaustive logo collection. The matrix is not a checklist of brands.

**Dependency rule:** tools that require crates live in non-published `fuzz/`, `xtask/`, or
`tests/` members, or run as external binaries. They never become runtime dependencies of
published crates.

### 8.3 Test gates by cadence

| Gate | Frequency | Required checks | Blocking |
|------|-----------|-----------------|----------|
| PR | Every push | fmt, strict Clippy, unit/integration/doc tests, compile-fail, Miri on core, Kani smoke on critical gadgets, 60 s fuzz smoke, cross-target check, zero-dependency check | Yes |
| Nightly | Daily | Full fuzzing (≥1 h), ASan/UBSan/TSan/MSan where supported, mutation testing, coverage report, Kani full suite, `cargo-hack` feature powerset, reproducible build, Valgrind | Yes for merge queue |
| Weekly | Weekly | Hardware TVLA/CPA, compiler matrix, platform matrix, red-team regression suite | Yes for release branch |
| Release | Every release | Full SAL-4 evidence bundle, external audit/red-team sign-off, signed artifacts, SBOM, reproducible build verification | Yes |
| Continuous | Ongoing | Long-running fuzzing, bug bounty, telemetry-free crash reports, dependency/toolchain monitoring | Advisory |

### 8.4 Evidence bundle

Every release publishes an evidence directory containing:

- Threat model, claims matrix, and non-claims.
- Test reports, coverage, mutation score, fuzz statistics and corpus hashes.
- Kani/model-checking proofs and assumptions.
- Constant-time assembly reports and dudect results.
- TVLA/CPA/DPA raw traces, acquisition parameters, analysis scripts, and target details.
- Red-team report, audit report, remediation tickets, and retest report.
- SBOM, dependency-policy report, build hashes, signatures, and exact reproduction commands.
- Known limitations and accepted risks.

**Evidence rule:** a claim is not "done" until the raw data, analysis code, tool versions,
target details, and commit hash are published and reproducible by an independent party.

### 8.5 Tool qualification and limitations

Each tool must have a documented scope and known limitations. Examples:

- Miri does not support FFI, OS syscalls, inline assembly, or all platform behavior.
- Kani/CBMC supports a subset of Rust and bounded models; it is not a full proof of the
  unbounded program.
- Sanitizers require nightly and, for MSan, instrumented libc; coverage varies by platform.
- TSan and loom cover different concurrency models; neither proves absence of all races.
- Fuzzing finds bugs, but cannot prove their absence.
- TVLA can produce false positives and false negatives; use multiple tests and a known-leaky
  control.
- Mutation testing measures test strength, not security.
- Coverage measures executed code, not correctness.

Any gap must be covered by another method or explicitly accepted in the risk register.

---

## 9. Red team and adversarial assurance

### 9.1 Independence and objectives

The red team must be independent of the implementation team. For a single-developer project,
this means an external contractor, an academic partner, or a separate security team. Internal
adversarial review is useful but cannot be the sole basis of a production claim.

The red team assumes Kerckhoffs's principle: the design, source, and documentation are known
to the attacker. Objectives:

- Recover keys or secret-dependent information from power, EM, or timing traces.
- Bypass masking through compiler optimization, share recombination, glitches, or API misuse.
- Cause fail-open behavior in entropy, RNG, error handling, or configuration.
- Exploit the CLI, trace parser, build system, or release pipeline.
- Cause denial of service or resource exhaustion.
- Find gaps between the documented claims and the actual implementation.

### 9.2 Scope and rules of engagement

**In scope:** `sicas-core`, `sicas-harness`, `sicas-cli`, reference integration, supported
platforms, build/CI/release pipeline, documentation and security claims, and the hardware
target used for leakage measurement.

**Out of scope:** third-party OS/toolchain/hardware vulnerabilities, real user data,
production systems, social engineering, and destructive physical attacks. Fault injection is
in scope only on dedicated lab hardware.

**Rules:** written authorization and scope document; isolated lab; synthetic keys and data;
defined stop conditions; secure data handling; emergency contact; safe-harbor terms; no
testing outside the agreed window.

### 9.3 Attack catalog

| Category | Tests | Success criterion |
|----------|-------|-------------------|
| Side-channel | SPA, DPA, CPA, template attacks, higher-order analysis, power and EM, trace alignment, multiple devices, temperature/voltage variation | Recover key material or distinguish secrets beyond the declared trace budget |
| Masking bypass | Compiler optimization at all levels, LTO, share recombination, glitch effects, declassification points, S-box indexing, RNG reuse/bias | Break the first-order masking claim or produce secret-dependent leakage |
| Timing | dudect, cache/branch/memory-access timing, early exits, variable-latency operations | Statistically distinguishable secret-dependent timing |
| Fault injection | Voltage/clock glitching, RNG stuck-at/bias, entropy starvation, truncated traces, malformed input, panic/unwind | Fail-open behavior, key disclosure, or unrecoverable corruption |
| API misuse | Secret in `Debug`/logs/errors/panics, `Clone`/`Eq`/`Index` misuse, serialization, zeroization gaps, declassification abuse | Extract a secret or bypass the type-level protections |
| Build and supply chain | Reproducible-build mismatch, artifact substitution, CI compromise, toolchain tampering, signed-tag verification, dev-tool dependency confusion | Ship or execute untrusted code, or produce a non-reproducible artifact |
| Parser and DoS | Huge inputs, integer overflow, allocation failure, infinite loops, pathological traces, CLI fuzzing | Crash, hang, memory exhaustion, or panic in a security path |
| Protocol and usage | Nonce reuse, key lifetime, error oracles, side-channel oracles, unsafe configuration | Recover keys or violate the security contract through normal use |
| Detection and response | Blue-team exercise: can the team detect, triage, and remediate an attack? | Missed critical finding or failed remediation/retest |

### 9.4 Cadence

- **Pre-v1.0:** full external red team engagement (4–6 weeks).
- **Every major release:** targeted red team regression (1–2 weeks).
- **Quarterly:** internal adversarial regression suite.
- **Annually:** full external red team plus security audit.
- **After v1.0:** scoped bug bounty with clear rules and a disclosure policy.

### 9.5 Severity and remediation SLA

| Severity | Examples | Remediation SLA | Release impact |
|----------|----------|-----------------|----------------|
| Critical | Key recovery, fail-open entropy, remote code execution | 7 days | Blocks release |
| High | Practical masking bypass, secret disclosure, build compromise | 30 days | Blocks release |
| Medium | Limited leakage, DoS, defense-in-depth gap | 90 days | Risk acceptance required |
| Low | Hardening, documentation, minor information leak | Next release | Tracked |

Use CVSS 4.0 plus side-channel-specific severity. Every finding requires a retest and a
public summary after remediation.

### 9.6 Deliverables and disclosure

- Red-team report with reproducible proof-of-concepts, traces, scripts, and affected versions.
- Remediation tickets with owners and deadlines.
- Retest report confirming fixes.
- Public security summary and CVE identifiers where appropriate.
- Coordinated disclosure timeline and credit policy.

---

## 10. Performance budgets (initial, to be validated)

| Component | Initial target | Notes |
|-----------|----------------|-------|
| Constant-time gadgets | ≤ 2× naive on x86_64; ≤ 3× on Cortex-M4 | Measure, do not assume |
| Masked ChaCha20 | ≤ 5× cycles/byte vs. unmasked; hard ceiling 10× | ARX is masking-friendly |
| Masked AES-128 (if added) | Set after prototype; full-scan S-box is expensive | Do not promise a number before measurement |
| Harness TVLA | 1M traces × 1k samples ≤ 60 s on a laptop | Optimize after correctness |
| Inactive runtime overhead | Zero background threads/work | No always-on daemon |
| Jitter | Bounded, configurable, off by default | Only if measured benefit |

---

## 11. Risk register

| ID | Risk | Likelihood | Impact | Mitigation |
|----|------|------------|--------|------------|
| R1 | Original power-cancellation pillar is invalid | High | Critical | Drop/reframe; do not claim |
| R2 | LLVM pass cannot meet safe Rust/zero deps | High | Critical | Typed API first; research gate |
| R3 | Zero deps forces home-grown crypto/entropy | Medium | High | Do not ship primitives; inject OS/host entropy; audit |
| R4 | Safe Rust blocks syscalls/zeroization/inline asm | High | Medium | Isolate unsafe in one optional crate or drop features |
| R5 | Compiler optimizations break masking | Medium | High | `black_box` barriers, assembly checks, per-toolchain evidence |
| R6 | Single developer cannot self-certify production | High | High | External audit, narrow claims, independent review |
| R7 | No hardware budget | High | High | Borrow/rent/partner; simulation is not production evidence |
| R8 | Clippy nursery instability | Medium | Low | Pin nightly, allowlist, scheduled upgrades |
| R9 | Scope creep | High | High | ADRs, phase gates, explicit non-goals |
| R10 | TVLA false negatives | Medium | High | Multiple tests, known-leaky control, independent re-analysis |
| R11 | Legal/ethical/dual-use concerns | Low | Medium | Responsible-use policy, disclosure process, export review |
| R12 | Verification-tool blind spots (Miri/Kani/sanitizers do not cover all code) | High | High | Layered methods, documented uncovered paths, independent review |
| R13 | Red-team scope gaps or false confidence | Medium | High | Independent team, attack catalog, retest, public evidence |
| R14 | CI/build supply-chain compromise | Medium | Critical | Pinned actions/toolchains, least privilege, signed artifacts, reproducible builds, SLSA |
| R15 | Fuzzing corpus blind spots | Medium | High | Structured fuzzing, coverage-guided campaigns, seed corpus, long runs |
| R16 | Performance overhead of hardening | High | Medium | Budgets, feature flags, per-target measurement |
| R17 | Superlative marketing overclaim | Medium | High | SAL-4 claim policy, security/legal review of all public claims |

---

## 12. Production release checklist

- [ ] Threat model approved; non-goals documented.
- [ ] Every claim has evidence and a non-claim.
- [ ] `sicas-core` forbids unsafe; any unsafe boundary reviewed and documented.
- [ ] Zero runtime dependencies verified by CI.
- [ ] Format, Clippy, tests, docs, Miri, ASan, UBSan, TSan/MSan (where supported), Valgrind,
      cross-target checks green.
- [ ] Kani/model-checking proofs pass for all critical invariants; assumptions documented.
- [ ] Mutation score ≥90% on core security logic; coverage report reviewed.
- [ ] Release-blocking fuzz campaign completed with no crashes; corpus archived.
- [ ] Compiler matrix (opt levels, LTO, codegen-units, panic strategy) verified.
- [ ] Platform matrix verified on QEMU and real hardware where applicable.
- [ ] TVLA/CPA/DPA/SPA/template/higher-order evidence on real hardware published with raw
      traces, multiple devices, and environmental variation.
- [ ] Fault-injection and RNG-failure tests completed where hardware allows.
- [ ] External audit complete; no open critical/high findings.
- [ ] Independent red team complete; no open critical/high findings; retest report published.
- [ ] SAL-4 evidence bundle published and independently reproducible.
- [ ] Reproducible build verified; artifacts signed; SBOM and SLSA provenance published.
- [ ] `SECURITY.md`, CVE process, disclosure policy, and bug-bounty plan published.
- [ ] User/integration/limitations docs complete.
- [ ] Performance budgets met or explicitly accepted.
- [ ] License and export/dual-use review complete.
- [ ] Release candidate publicly reviewed.

---

## 13. Decisions needed before coding

1. **Scope:** approve the revised v1.0 (core + harness + reference integration), or keep the
   original three pillars as a research prototype with no production claim.
2. **Unsafe boundary:** allow one small, audited `sicas-platform` crate for OS entropy,
   zeroization, and affinity, or require 100% safe Rust and drop those features.
3. **Dependencies:** confirm "zero runtime dependencies" and decide whether dev-only tools may
   use crates in non-published members.
4. **Hardware:** is there access to a ChipWhisperer/oscilloscope/glitcher or a lab partner?
   Without it, v1.0 cannot claim physical validation.
5. **Audit and red team:** is there budget/partner for an independent audit and red-team
   engagement? Without it, v1.0 is "beta", not SAL-4.
6. **Assurance target:** confirm SAL-4 as the v1.0 target, or choose a lower level with an
   honest label.
7. **Formal verification:** which critical invariants must be model-checked (mask correctness,
   panic-freedom, bounds, constant-time logic), and which tools are acceptable
   (Kani/CBMC, Creusot, Prusti, Verus)?
8. **Bug bounty:** is there budget/scope for a post-v1.0 bounty, or only coordinated
   disclosure?
9. **Reference algorithm:** ChaCha20 first (recommended), AES-128 first, or both.
10. **Platform priority:** Linux x86_64 tooling + Cortex-M4 measurement, or another target.

---

## 14. Next 10 working days

1. Approve or reject the revised scope and SAL-4 target; record the decision in an ADR.
2. Write the threat model, claims matrix, and red-team scope/authorization document.
3. Write ADRs for unsafe boundary, zero deps, dropped eBPF/LLVM pillars, entropy policy,
   SAL-4, and the verification-tool qualification matrix.
4. Scaffold the workspace with `sicas-core` and `sicas-harness`; pin the toolchain.
5. Add CI for fmt, strict Clippy, tests, docs, Miri, sanitizers, Kani smoke, fuzz smoke,
   mutation testing, and the zero-dependency check.
6. Implement `Choice`, `ct_eq`, `ct_select`, `Secret<T>`, and `Masked<u32>` with unit,
   property, compile-fail, and Kani tests.
7. Build the synthetic TVLA harness, a known-leaky control target, and the fuzz corpus.
8. Start hardware, audit, and independent red-team partner conversations.
9. Hold the Phase 0 gate review; do not start Phase 1 until it passes.

---

## 15. Bottom line

SiCaS can become credible production security software, but not by implementing the current
document literally. The defensible product is a **safe Rust masking and leakage-assessment
toolkit with measured claims**, plus optional runtime adapters.

The v1.0 bar is **SAL-4**: layered static/dynamic verification, Miri and sanitizers, Kani/model
checking, mutation testing, continuous fuzzing, compiler/platform matrices, real-hardware
TVLA/CPA/DPA/SPA/template/higher-order testing, fault injection, an independent red team, an
external audit, reproducible signed builds, and a public evidence bundle. No release may claim
SAL-4 until every criterion is met and independently verified.

The compiler and kernel ideas remain valuable research directions, but they need feasibility
gates and must not be marketed as solved until they are proven.
