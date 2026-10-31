# Audit Technique Matrix

This maps the techniques an independent reviewer would normally use to the local
evidence. It is a self-audit record, not an independent review.

| Technique | Local execution | Evidence |
|---|---|---|
| Threat modeling | Assets, adversary, non-goals | `docs/THREAT_MODEL.md` |
| Claims review | Claim/non-claim matrix | `docs/SECURITY_CLAIMS.md` |
| Source review | Adversarial line review and fixes | `docs/AUDIT_REPORT_2026-10-30.md`, `docs/SELF_AUDIT_2026-10-31.md` |
| Static analysis | Strict Clippy, restriction lints, rustdoc | full proof logs |
| Memory safety | Miri, ASan, Valgrind | full proof logs |
| Formal/model checking | 30 Kani harnesses | full proof logs |
| Differential testing | RFC 8439 and unmasked reference | `sicas-reference` tests |
| Fuzzing | 5 × 1M campaigns including bitsliced batch | `docs/FUZZ_EVIDENCE.md` |
| Mutation testing | 542 caught, 0 missed, 51 unviable | `docs/MUTATION_EVIDENCE.md` |
| Coverage | 97.20% lines, 94.77% regions | full proof logs |
| Constant-time inspection | 69-configuration assembly matrix | `evidence/ct/asm-2026-10-31.md` |
| Timing analysis | dudect debug and release profiles | `docs/CT_EVIDENCE.md` |
| Leakage modeling | First-order checks with negative control | `evidence/leakage/2026-10-31-x86_64.md` |
| DPA | Single-bit difference-of-means on AES S-box output; synthetic key recovery | `crates/sicas-harness/src/dpa.rs` tests |
| Fault injection | Entropy failure, final-mask failure, counter wrap | `crates/sicas-reference/src/chacha20.rs` tests |
| Parser/DoS | Fuzzing and trace limits | `fuzz/fuzz_targets/trace_parser.rs` |
| Supply chain | Zero dependencies, audit, deny, SBOM | `docs/PROOF_OBLIGATIONS.md` |
| Reproducibility | Three-build hash comparison | `evidence/release/2026-10-31-x86_64.md` |
| Release validation | Manifest, SBOM, provenance, signing dry run | `docs/RELEASE.md` |
| Self-audit | Findings and fixes A-01 to A-12 | audit reports |
| Zeroization | Experimental volatile-write adapter | `crates/sicas-zeroize`; unaudited |

## Not locally executable

- Independent attestation: an auditor cannot be independent of the implementer when
  both are the same local process.
- Physical power/EM traces: no target board or acquisition hardware.
- Trusted production signing: no production identity.

These are the only items that remain outside the local technique matrix.
