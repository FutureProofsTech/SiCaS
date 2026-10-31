# SAL-4 Local Readiness

This matrix separates what the local software proof can establish from what requires
independent parties or physical hardware. It is a readiness record, not a SAL-4 claim.

Current public label: **SAL-2 locally**.

| SAL-4 criterion | Local evidence | Local status |
|---|---|---|
| Unit, integration, property, and compile-fail tests | Workspace tests, doctests, compile-fail cases | Complete |
| Memory safety and UB checks | Miri, ASan, Valgrind | Complete locally |
| Constant-time evidence | Dudect timing, 69-configuration assembly matrix | Complete for inspected targets/configurations |
| Formal/model checking | 30 Kani harnesses for core gadgets, masking, refresh, addition, and share uniformity | Complete for declared models |
| Fuzzing | Four 1M campaigns; 10M on masked_ops/ct_lookup/trace_parser and 1M on ChaCha20 in the long campaign | Complete locally |
| Mutation and coverage | 0 missed viable mutants; 96.90% lines | Complete locally |
| Masking model | Deterministic first-order checks with negative control; ChaCha20 key-share check | Complete in the model |
| Cross-architecture functional evidence | AArch64 QEMU, i686 native, four-target assembly checks | Complete locally |
| Reproducible release | Three-build hash comparison, SBOM, provenance, manifest validator | Complete locally |
| Signing process | Ephemeral test-key signing and verification dry run | Process proven; production identity not established |
| Zeroization | Design review and adapter checklist | Not implemented |
| Performance | Masked ChaCha20 benchmark; see [`PERFORMANCE_EVIDENCE.md`](PERFORMANCE_EVIDENCE.md) | Hard ceiling met: bitsliced batch 7.15× vs unmasked in the final full proof; 5× target open |
| Physical power/EM TVLA/CPA/DPA | None | Not software-local |
| ARM/RISC-V runtime timing on hardware | None | Not software-local |
| Independent adversarial review | Local adversarial audit only | Not software-local |

## Evidence beyond SAL-2

The local package already contains several SAL-3 prerequisites:

- 30 Kani harnesses for core masking and constant-time gadgets;
- self-audit with a fixed batch state-consistency defect;
- first-order leakage model with a negative control;
- 64-lane bitsliced masking and lane-independent addition tests;
- five 1M-execution fuzz campaigns, including the bitsliced batch;
- 69-configuration assembly matrix and release-profile timing.

Those are necessary but not sufficient for SAL-3. SAL-3 still requires real-hardware
TVLA and independent audit; SAL-4 adds independent red team and production signing.

## Interpretation

Every criterion that can be proven with local software and the available host is green,
except the documented 5× performance target. The experimental `sicas-zeroize` adapter
is implemented but unaudited; the remaining SAL-4 criteria require physical targets or
independent parties and cannot be truthfully completed by local software alone.

The project therefore remains **SAL-2 locally**. A SAL-4 claim requires the external
criteria to be completed and independently verified.
