# Tooling Inventory

| Tool | Version used | Purpose |
|---|---|---|
| Rust stable | 1.99.0 | build, tests, MSRV |
| Rust nightly | nightly-2026-08-03 | fmt, Clippy, Miri, sanitizers |
| Kani / CBMC | 0.68.0 / 6.11.0 | model checking |
| cargo-fuzz / libFuzzer | 0.13.2 | fuzzing |
| cargo-mutants | 27.1.0 | mutation testing |
| cargo-llvm-cov | 0.9.1 | coverage |
| cargo-audit | 0.22.2 | advisory scanning |
| cargo-deny | 0.20.2 | license, source, ban, advisory policy |
| Miri | nightly component | undefined behavior |
| AddressSanitizer | nightly `-Zsanitizer=address` | memory errors |
| Python | 3.x | release manifest, manifest validator, evidence bundler, and assembly checker |

## Policy

- Versions are pinned in `rust-toolchain.toml` or documented here.
- A tool is adopted only for a documented property; its limitations are recorded.
- Tool results are published with the commit, target, and exact command.
- Missing tool coverage is recorded in `docs/PROOF_OBLIGATIONS.md`.
- Tool installation and execution are not runtime dependencies of the published
  crates.
