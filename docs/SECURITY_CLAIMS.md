# Security Claims

| ID | Claim | Mechanism | Evidence required | Status |
|----|-------|-----------|-------------------|--------|
| C1 | The safe API prevents accidental secret-dependent branching/indexing | `Secret<T>` has no `PartialEq`, `Ord`, `Hash`, `Index`, `Clone`, or `Copy` | Compile-fail tests, API review | In progress: compile-fail doctests pass |
| C2 | Constant-time gadgets are data-independent on supported targets | Branchless bitwise formulas | Assembly inspection, dudect-style tests | In progress: dudect 5×1M debug and 3×1M release trials pass on x86_64 (max abs t = 1.56); assembly matrix passes 69 configurations across opt levels, codegen units, panic strategies, and x86_64 thin/fat LTO; other architectures/LTO combinations pending |
| C3 | First-order Boolean masking reduces first-order DPA below the TVLA threshold for the reference algorithm | Two-share masking with fresh randomness and per-block state refresh | TVLA report, raw traces, target details | In progress: masked ChaCha20 passes RFC 8439 and differential tests; bitsliced batch matches the independent reference and passes lane-independent addition tests; deterministic first-order model checks pass with a detected negative control; hardware TVLA pending |
| C4 | Entropy failure is fail-closed | `Result`-based entropy API, no weak fallback | Fault-injection tests, code review | In progress: unit failure test passes; Unix `/dev/urandom` adapter tested; other platforms fail closed |
| C5 | Zero runtime dependencies | Workspace policy, CI dependency check | `cargo metadata`/`cargo tree` report | Verified locally; CI pending |
| C6 | No unsafe code in core logic crates | `#![forbid(unsafe_code)]` | CI, source audit | Verified locally |
| C7 | Builds are reproducible | Pinned toolchain, locked dependencies, deterministic flags | Two-build hash comparison | Verified locally: identical SHA-256 across independent target directories; CI/SLSA run pending |
| C8 | Critical invariants are model-checked | Kani/CBMC, Creusot/Prusti/Verus where applicable | Proof artifacts, assumptions, uncovered paths | In progress: 30 Kani harnesses pass for core gadgets, including every masked linear and logical operation, addition, mask refresh, share reconstruction, and share-uniformity coverage |
| C9 | v1.0 meets SAL-4 | Layered verification, red team, audit, evidence bundle | Independent verification of every SAL-4 criterion | Pending |
| C10 | Release artifacts are signed and traceable | Sigstore/minisign, SLSA provenance, SBOM | Signature and provenance verification report | Pending |

## Non-claims

No protection against higher-order DPA, fault injection, invasive attacks, all
algorithms, all platforms, all compilers, or all side channels. No "zero leakage"
claim. No throughput or production-performance claim: the masked reference currently
exceeds the declared performance budget. No production zeroization claim: the
experimental adapter is unaudited. The public claim is "SAL-4 under threat model
T" once every SAL-4 criterion is met and independently verified.
