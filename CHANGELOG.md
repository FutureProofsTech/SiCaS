# Changelog

## Unreleased

### Changed

- Relicensed the project from MIT OR Apache-2.0 to GPL-3.0-only.

### Added

- `sicas-core` safe, `no_std`, zero-dependency constant-time primitives.
- Two-share Boolean masking for `u8`, `u16`, `u32`, `u64`, and `u128`.
- Fail-closed entropy-source abstraction.
- Kani proof harnesses for core constant-time and masking invariants.
- libFuzzer targets for masked operations, constant-time lookup, and trace parsing.
- AddressSanitizer, Miri, strict Clippy, and mutation-testing gates.
- `sicas-harness` with strict binary trace format, TVLA, CPA, synthetic trace
  generation, dudect-style timing tooling, and a CLI.
- `sicas-reference` with a first-order Boolean masked ChaCha20 keystream block,
  RFC 8439 validation, and differential tests against an unmasked reference.
- `sicas-harness ct` with a dudect-style suite for constant-time primitives,
  Markdown evidence reports, and an assembly inspection example.
- `tools/ct_asm_check.py` for multi-architecture assembly checks, with evidence for
  x86_64, AArch64, Thumbv7em, and RISC-V.
- CI coverage, supply-chain, and reproducible-build gates.
- Four 1M-execution fuzz campaigns with published evidence.
- `sicas-platform` safe OS entropy adapter with fail-closed behavior.
- Persistent project instructions and working agreement for local-first, autonomous
  proof work, with external review reserved for a second opinion after local proof.
- Trace preprocessing: trim, centered moving average, and normalized cross-correlation
  alignment.
- Generic CPA leakage models via `cpa_with_model`.
- Audit fixes: independent initial-state masks for masked ChaCha20, per-block mask
  refresh, overflow-safe moving average, and stricter empty-trace serialization.
- AArch64 Linux functional test runner using QEMU; 91 workspace library tests pass.
- 32-bit x86 Linux functional tests; 102 all-target tests pass.
- Reproducible release binary evidence with matching SHA-256 across independent target
  directories.
- `tools/local_proof.sh` quick/full local proof runner with logs and summary.
- Post-local-proof backlog for remaining local hardening before external review.
- First-order leakage-model checker with a negative control, plus a masked
  ChaCha20 key-share correlation test.
- Kani proofs for every masked linear and logical operation across all integer
  widths, including `shares`, refresh, select, equality, and bit extraction.
- Property tests for trace trimming, filtering, alignment, CPA recovery, and TVLA
  tie/threshold behavior.
- `Masked::shares` accessor for explicit leakage-analysis boundaries.
- Release-manifest validator with checksum, provenance, SBOM, workspace, and commit
  consistency tests.
- Zeroization design review and adapter checklist.
- Compiler-matrix assembly evidence: 69 configurations across opt levels, codegen units,
  panic strategies, and x86_64 thin/fat LTO.
- First-order DPA difference-of-means tool over the AES S-box output, with synthetic key-recovery tests.
- White paper and yellow paper in LaTeX with PDF builds and a document version log.
- Dedicated `chacha20_batch` fuzz target for the 64-block bitsliced path.
- Experimental `sicas-zeroize` crate with volatile byte writes, compiler fence, and
  redacted `ZeroizingBytes`; isolated, non-published, and unaudited.
- Valgrind Memcheck evidence: four workspace library binaries, zero errors.
- Local signing dry run with an ephemeral test key: detached signature produced and
  verified; production identity remains open.
- Release-profile dudect evidence: 3×1M samples per check, max |t| = 2.1196.
- Differential statistics validation: 200 deterministic cases match an independent
  Python implementation.
- Bitsliced masked ChaCha20 batch: 64 blocks per mask stream and a fast 64×64
  bit-matrix transpose. The measured overhead is 7.26× vs unmasked, meeting the
  10× hard ceiling; the ≤5× target remains open.
- Full local proof at commit `8c41e4b`: all security gates pass, including
  full-workspace Miri, 30 Kani harnesses, 130 ASan tests, zero missed viable mutants,
  97.20% line coverage, five 1M-execution fuzz campaigns, cross-architecture tests,
  release validation, reproducible-build comparison, and the performance gate at
  7.26×. The 10× hard ceiling is met; the 5× target remains open.
- Release manifest/SBOM/provenance tooling, Sigstore/SLSA release workflow, and
  audit/red-team/release documentation.
- Threat model, security claims, proof obligations, code-quality standard, assurance
  program, support matrix, and evaluation guide.

### Status

- SAL-2 locally.
- Not audited, not hardware-validated, and not safe for production secrets.
