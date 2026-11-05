# Security Claims

| ID | Claim | Mechanism | Evidence required | Status |
|----|-------|-----------|-------------------|--------|
| C1 | The safe API prevents accidental secret-dependent branching/indexing | `Secret<T>` has no `PartialEq`, `Ord`, `Hash`, `Index`, `Clone`, or `Copy` | Compile-fail tests, API review | In progress: compile-fail doctests pass |
| C2 | The reviewed constant-time comparison/selection functions are branch-free on the inspected configurations | Branchless bitwise formulas; public loop bounds; no secret branch or index | Assembly inspection, dudect-style tests, source review | In progress: dudect 5×1M debug and 3×1M release trials pass on x86_64 (max abs t = 1.56); the 69-configuration assembly matrix covers `ct::eq_u32`, `ct::select_u32`, and `ct::is_zero_u32`. `ct::lookup_u8` intentionally contains a public 256-iteration loop; `Masked::add_masked` is covered by dudect and functional tests, not the assembly matrix; the bitsliced ChaCha20 reference is covered by source review, differential tests, and a targeted `masked_word_planes` source + `opt-level=0` `setne` gate (`tools/reference_ct_check.py`), not by the full assembly matrix. Other architectures/LTO combinations and a full reference matrix remain pending |
| C3 | First-order Boolean masking reduces first-order DPA below the TVLA threshold for the reference algorithm | Two-share masking with fresh randomness and per-block state refresh | TVLA report, raw traces, target details | In progress: masked ChaCha20 passes RFC 8439 and differential tests; bitsliced batch matches the independent reference and passes lane-independent addition tests; deterministic first-order model checks pass with a detected negative control; hardware TVLA pending |
| C4 | Entropy failure is fail-closed | `Result`-based entropy API, no weak fallback | Fault-injection tests, code review | In progress: unit failure test passes; Unix `/dev/urandom` adapter tested; other platforms fail closed |
| C5 | Zero runtime dependencies | Workspace policy, CI dependency check | `cargo metadata`/`cargo tree` report | Verified locally; CI pending |
| C6 | No unsafe code in core logic crates | `#![forbid(unsafe_code)]` | CI, source audit | Verified locally |
| C7 | Builds are reproducible | Pinned toolchain, locked dependencies, deterministic flags | Two-build hash comparison | Verified locally: identical SHA-256 across independent target directories; CI/SLSA run pending |
| C8 | Core masked-gadget invariants are model-checked | Kani/CBMC, Creusot/Prusti/Verus where applicable | Proof artifacts, assumptions, uncovered paths | In progress: 30 Kani harnesses pass for core gadgets, including every masked linear and logical operation, addition, mask refresh, share reconstruction, and share-uniformity coverage. The bitsliced ChaCha20 composition is validated by differential and lane tests, not Kani; no claim is made that the composition is model-checked |
| C9 | v1.0 meets SAL-4 | Layered verification, red team, audit, evidence bundle | Independent verification of every SAL-4 criterion | Pending |
| C10 | Release artifacts are signed and traceable | Sigstore/minisign, SLSA provenance, SBOM | Signature and provenance verification report | Pending |

## Non-claims

No protection against higher-order DPA, fault injection, invasive attacks, all
algorithms, all platforms, all compilers, or all side channels. No "zero leakage"
claim. No throughput or production-performance claim: the masked reference currently
exceeds the declared performance budget. No production zeroization claim: the
experimental adapter is unaudited. The public claim is "SAL-4 under threat model
T" once every SAL-4 criterion is met and independently verified.
