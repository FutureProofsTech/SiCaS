# Security Claims

| ID | Claim | Mechanism | Evidence required | Status |
|----|-------|-----------|-------------------|--------|
| C1 | The safe API prevents accidental secret-dependent branching/indexing | `Secret<T>` has no `PartialEq`, `Ord`, `Hash`, `Index`, `Clone`, or `Copy` | Compile-fail tests, API review | In progress: compile-fail doctests pass |
| C2 | Constant-time gadgets are data-independent on supported targets | Branchless bitwise formulas | Assembly inspection, dudect-style tests | In progress: dudect 5×1M trials pass on x86_64; assembly checker passes for x86_64, AArch64, Thumbv7em, and RISC-V at opt-level 0; release/other levels pending |
| C3 | First-order Boolean masking reduces first-order DPA below the TVLA threshold for the reference algorithm | Two-share masking with fresh randomness | TVLA report, raw traces, target details | In progress: masked ChaCha20 passes RFC 8439 and differential tests; hardware TVLA pending |
| C4 | Entropy failure is fail-closed | `Result`-based entropy API, no weak fallback | Fault-injection tests, code review | In progress: unit failure test passes; Unix `/dev/urandom` adapter tested; other platforms fail closed |
| C5 | Zero runtime dependencies | Workspace policy, CI dependency check | `cargo metadata`/`cargo tree` report | Verified locally; CI pending |
| C6 | No unsafe code in core logic crates | `#![forbid(unsafe_code)]` | CI, source audit | Verified locally |
| C7 | Builds are reproducible | Pinned toolchain, locked dependencies, deterministic flags | Two-build hash comparison | Pending |
| C8 | Critical invariants are model-checked | Kani/CBMC, Creusot/Prusti/Verus where applicable | Proof artifacts, assumptions, uncovered paths | In progress: 14 Kani harnesses pass for core gadgets, including masked addition |
| C9 | v1.0 meets SAL-4 | Layered verification, red team, audit, evidence bundle | Independent verification of every SAL-4 criterion | Pending |
| C10 | Release artifacts are signed and traceable | Sigstore/minisign, SLSA provenance, SBOM | Signature and provenance verification report | Pending |

## Non-claims

No protection against higher-order DPA, fault injection, invasive attacks, all
algorithms, all platforms, all compilers, or all side channels. No "zero leakage"
claim. The public claim is "SAL-4 under threat model T" once every SAL-4 criterion is
met and independently verified.
