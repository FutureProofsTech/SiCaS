# Proof Obligations

No proof, no claim. Every entry in [`SECURITY_CLAIMS.md`](SECURITY_CLAIMS.md) links to
evidence here. Missing evidence downgrades or removes the claim.

| Property | Proof obligation | Primary tools | Evidence | Current status |
|----------|------------------|---------------|----------|----------------|
| Memory safety / no UB | No out-of-bounds, use-after-free, aliasing violation, or uninitialized read | Miri, ASan, UBSan, Valgrind | Miri report; sanitizer report | Miri and ASan passing locally; UBSan/Valgrind CI pending |
| Panic-freedom and bounds | No panic, overflow, or slice-index failure on untrusted input | Kani, fuzzing, restriction lints, `-C overflow-checks` | Proofs, fuzz stats, test reports | 14 Kani harnesses pass; four 1M-execution fuzz campaigns pass; continuous fuzzing pending |
| Functional correctness | Matches independent references and test vectors | Unit/integration tests, differential tests, Kani | Test logs and vectors | Core and harness tests pass; masked ChaCha20 matches RFC 8439 and the unmasked reference; Kani passes |
| Test-suite strength | No surviving mutants in security-critical code | `cargo-mutants` | Mutation report | Core: 111 caught, 0 missed, 20 unviable. Harness: 261 caught, 19 missed, 9 unviable (93.2% of viable mutants). Reference: 6 caught, 0 missed. Workspace: 378 caught, 19 missed, 29 unviable (95.2% of viable mutants) |
| Coverage | Line coverage stays above 90% | `cargo-llvm-cov` | Coverage report | 96.59% lines, 94.24% regions locally |
| Constant time | Execution is independent of secret values | dudect, ctgrind, assembly inspection | Timing reports and disassembly | Dudect 5×1M trials pass on x86_64 (max abs t = 1.67); assembly checker passes for x86_64, AArch64, Thumbv7em, and RISC-V at opt-level 0; release/other levels pending |
| Masking correctness | Shares combine correctly; no first-order leakage in the model | Property tests, Kani, TVLA/CPA | Proofs and raw traces | Property tests and Kani pass for all widths; TVLA pending |
| Entropy fail-closed | Entropy failure cannot silently produce predictable masks | Fault injection, unit tests, code review | Failure-injection report | Unit failure test passing; Unix `/dev/urandom` adapter tested; hardware/fault injection pending |
| Parser robustness | Malformed or hostile input cannot crash or hang the tool | Fuzzing, Miri, sanitizers, timeouts | Fuzz corpus and crash-free campaign | Trace parser passes a 1M-execution campaign; continuous fuzzing pending |
| API misuse resistance | Forbidden secret operations do not compile | Compile-fail tests, independent review | Compile-fail suite and review record | Compile-fail doctests passing |
| Supply-chain integrity | No hidden runtime dependencies; artifacts are reproducible | `cargo-audit`, `cargo-deny`, `cargo-vet`, SBOM, reproducible builds | Dependency and build reports | Zero third-party deps; `cargo audit`/`cargo deny` pass; reproducible binary hash match locally |
| Performance | Overhead stays within the declared budget | `cargo-bloat`, `iai-callgrind`, `perf`, Callgrind | Benchmark reports | Budgets not yet measured |
| Documentation | Every public item is documented and links resolve | `rustdoc -D warnings`, `missing_docs` | Doc build log | `missing_docs` enforced; CI pending |
| Release integrity | Artifacts are signed, traceable, and reproducible | Sigstore/minisign, SLSA provenance, SBOM | Signature and provenance verification | Not implemented |

The harness CLI (`crates/sicas-harness/src/main.rs`) is excluded from the mutation score.
It is covered by CLI integration tests; mutations of its branchless timing-input
selection are behaviorally equivalent for the constant-time property being measured.

## Evidence rule

A claim is not done until the raw data, analysis code, tool versions, target details,
and commit hash are published and reproducible by an independent party.
