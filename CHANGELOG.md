# Changelog

## Unreleased

### Added

- `sicas-core` safe, `no_std`, zero-dependency constant-time primitives.
- Two-share Boolean masking for `u8`, `u16`, `u32`, `u64`, and `u128`.
- Fail-closed entropy-source abstraction.
- Kani proof harnesses for core constant-time and masking invariants.
- libFuzzer targets for masked operations, constant-time lookup, and trace parsing.
- AddressSanitizer, Miri, strict Clippy, and mutation-testing gates.
- `sicas-harness` with strict binary trace format, TVLA, CPA, synthetic trace
  generation, dudect-style timing tooling, and a CLI.
- Threat model, security claims, proof obligations, code-quality standard, assurance
  program, support matrix, and evaluation guide.

### Status

- SAL-2 locally.
- Not audited, not hardware-validated, and not safe for production secrets.
