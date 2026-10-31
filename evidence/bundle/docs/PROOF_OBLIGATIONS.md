# Proof Obligations

No proof, no claim. Every entry in [`SECURITY_CLAIMS.md`](SECURITY_CLAIMS.md) links to
evidence here. Missing evidence downgrades or removes the claim.

| Property | Proof obligation | Primary tools | Evidence | Current status |
|----------|------------------|---------------|----------|----------------|
| Memory safety / no UB | No out-of-bounds, use-after-free, aliasing violation, or uninitialized read | Miri, ASan, UBSan, Valgrind | Miri report; sanitizer report | Full-workspace Miri passes at `8c41e4b`; 141-test ASan and five-binary Valgrind Memcheck pass at the DPA commit; Miri uses reduced iteration counts and skips native-only statistical/OS/subprocess paths; UBSan is unavailable in the pinned nightly, and MSan/TSan remain pending |
| Panic-freedom and bounds | No panic, overflow, or slice-index failure on untrusted input | Kani, fuzzing, restriction lints, `-C overflow-checks` | Proofs, fuzz stats, test reports | 30 Kani harnesses pass; five 1M-execution fuzz campaigns plus a long campaign (10M on three fast targets, 1M on ChaCha20) pass; continuous fuzzing pending |
| Functional correctness | Matches independent references and test vectors | Unit/integration tests, differential tests, Kani, QEMU | Test logs and vectors | Core and harness tests pass on x86_64, i686, and AArch64 QEMU; masked ChaCha20 matches RFC 8439 and the unmasked reference; statistics match an independent Python implementation over 200 deterministic cases; DPA recovers a synthetic AES S-box key from a single-bit leak; Kani passes |
| Test-suite strength | No surviving mutants in security-critical code | `cargo-mutants` | Mutation report | Core: 112 caught, 0 missed, 19 unviable. Harness: 357 caught, 0 missed, 23 unviable (100% of viable mutants). Reference: 69 caught, 0 missed, 8 unviable. Platform: 2 caught, 0 missed, 1 unviable. Workspace: 542 caught, 0 missed, 51 unviable (100% of viable mutants) |
| Coverage | Line coverage stays above 90% | `cargo-llvm-cov` | Coverage report | 97.23% lines, 94.82% regions in the full proof; 97.28% lines, 94.89% regions at the DPA commit |
| Constant time | Execution is independent of secret values | dudect, ctgrind, assembly inspection | Timing reports and disassembly | Dudect 5×1M debug-profile trials and 3×1M release-profile trials pass on x86_64 (max abs t = 2.25 debug, 1.56 release); assembly matrix passes 69 configurations: opt-level 0 on four targets, opt-levels 0/1/2/3/s and codegen/panic variants on x86_64 and AArch64, and thin/fat LTO on x86_64 |
| Masking correctness | Shares combine correctly; no first-order leakage in the model | Property tests, Kani, statistical model checks, TVLA/CPA | Proofs and raw traces | Property tests and Kani pass for all widths and operations; ChaCha20 refreshes working and initial state per block; deterministic first-order model checks pass with a detected negative control; hardware TVLA pending |
| Entropy fail-closed | Entropy failure cannot silently produce predictable masks | Fault injection, unit tests, code review | Failure-injection report | Unit failure test passing; Unix `/dev/urandom` adapter tested; hardware/fault injection pending |
| Parser robustness | Malformed or hostile input cannot crash or hang the tool | Fuzzing, Miri, sanitizers, timeouts | Fuzz corpus and crash-free campaign | Trace parser passes a 1M-execution campaign; continuous fuzzing pending |
| API misuse resistance | Forbidden secret operations do not compile | Compile-fail tests, independent review | Compile-fail suite and review record | Compile-fail doctests passing |
| Supply-chain integrity | No hidden runtime dependencies; artifacts are reproducible | `cargo-audit`, `cargo-deny`, `cargo-vet`, SBOM, reproducible builds | Dependency and build reports | Zero third-party deps; `cargo audit`/`cargo deny` pass; reproducible binary hash match locally |
| Performance | Overhead stays within the declared budget | `cargo-bloat`, `iai-callgrind`, `perf`, Callgrind | Benchmark reports | Bitsliced batch masked ChaCha20 measures 7.15× vs unmasked in the final full proof (7.26× in an earlier run); the 10× hard ceiling is met, the ≤5× target is not. See `docs/PERFORMANCE_EVIDENCE.md` |
| Documentation | Every public item is documented and links resolve | `rustdoc -D warnings`, `missing_docs` | Doc build log | `missing_docs` enforced; CI pending |
| Volatile zeroization | Secret bytes are erased before memory reuse | Volatile writes, compiler fence, independent review | Source, tests, assembly review | Experimental `sicas-zeroize` adapter passes tests and x86_64 assembly inspection; independent review pending |
| Release integrity | Artifacts are signed, traceable, and reproducible | Sigstore/minisign, SLSA provenance, SBOM, manifest validator | Signature and provenance verification | Manifest, SBOM, provenance, and consistency validator pass in the full local proof; local test-key signing dry run passes; production signing/SLSA pending |

The harness CLI (`crates/sicas-harness/src/main.rs`) is excluded from the mutation score.
It is covered by CLI integration tests; mutations of its branchless timing-input
selection are behaviorally equivalent for the constant-time property being measured.

## Evidence rule

A claim is not done until the raw data, analysis code, tool versions, target details,
and commit hash are published and reproducible by an independent party.
