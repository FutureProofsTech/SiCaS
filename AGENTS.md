# SiCaS Project Instructions

These instructions are persistent project memory. Read them at the start of every
session in this repository.

## Mission

Build SiCaS as production-grade side-channel attenuation and leakage-assessment
software in **safe Rust** with **zero third-party runtime dependencies**. Do all
engineering, verification, and evidence work locally first. External audit, independent
red team, hardware leakage validation, and production signing happen only after the
local proof package is complete.

## Non-negotiable rules

- Safe Rust only. Published crates must use `#![forbid(unsafe_code)]`.
- Zero third-party runtime dependencies. Workspace dependencies are allowed.
- Never fabricate evidence. If a gate needs hardware, credentials, or an independent
  party, record it in `docs/GAP_REPORT.md` and continue with other local work.
- Never weaken or delete a test to make a gate pass. Fix the implementation.
- Every change needs tests, documentation, evidence, and a local Git commit.
- Keep claims precise and bounded. No "unbreakable", "zero leakage", or unverifiable
  superlatives.
- The current public label is **SAL-2 locally**. Do not claim production/SAL-4 until
  every external gate is complete and independently verified.
- Do not ask the user for routine confirmation or scope approval. Proceed autonomously
  with documented assumptions. Only pause for destructive actions, missing credentials,
  or physical hardware that cannot be simulated honestly.
- Continue through the local backlog. Do not stop while a local, testable task remains.
  If blocked, document the gap and move to the next local task.
- **Local proof before external review.** Do not submit anything for external audit or
  red team until every locally provable claim is proven, every gate is green, and the
  evidence bundle is complete. External review is a second opinion, not a substitute
  for local proof. Never use an external reviewer to discover basic defects that local
  testing, audit, or red-team simulation can find.
- If a claim can only be proven with hardware, credentials, or an independent party,
  state that explicitly and keep it out of the “locally proven” set. Do not disguise a
  gap as a completed proof.

## Required quality gates

```bash
cargo +nightly-2026-08-03 fmt --all -- --check

cargo +nightly-2026-08-03 clippy \
  --workspace --all-targets --all-features --locked \
  -- -D warnings -D clippy::pedantic -D clippy::nursery

cargo test --workspace --all-targets --all-features --locked
cargo test --workspace --doc --locked
cargo +nightly-2026-08-03 miri test --workspace
cargo kani --workspace
RUSTFLAGS="-Zsanitizer=address" cargo +nightly-2026-08-03 test --workspace --lib
cargo mutants --workspace --no-shuffle
cargo llvm-cov --workspace --all-features --fail-under-lines 90
cargo audit
cargo deny check
python3 tools/ct_asm_check.py
python3 tools/evidence_bundle.py --output evidence/bundle
```

The canonical local proof command is:

```bash
./tools/local_proof.sh quick   # fast gate set
./tools/local_proof.sh full    # full local proof, 20+ minutes
```

Also run the four fuzz targets when touching core, trace, or reference code:

```bash
cargo +nightly-2026-08-03 fuzz run masked_ops -- -runs=1000000
cargo +nightly-2026-08-03 fuzz run ct_lookup -- -runs=1000000
cargo +nightly-2026-08-03 fuzz run trace_parser -- -runs=1000000
cargo +nightly-2026-08-03 fuzz run chacha20 -- -runs=1000000
```

## Workflow for every phase

1. Read `PLAN.md`, `AGENTS.md`, `docs/GAP_REPORT.md`, and
   `docs/PROOF_OBLIGATIONS.md`.
2. Pick the highest-value local gap.
3. Implement it with tests, docs, and evidence.
4. Run the full gates.
5. Update the claims, proof obligations, gap report, and evidence bundle.
6. Commit to local Git.
7. Continue to the next local gap.

After the full local proof passes, continue with `docs/POST_PROOF_BACKLOG.md`. Do not
treat a passing local proof as the end of the project: freeze the local proof, publish
the evidence bundle, and only then proceed to external audit, red team, hardware
validation, and production signing.

## Repository map

- `crates/sicas-core` — constant-time primitives, `Secret`, two-share `Masked`,
  masked addition.
- `crates/sicas-reference` — masked ChaCha20 reference integration.
- `crates/sicas-harness` — trace format, preprocessing, TVLA, CPA, timing, CLI.
- `crates/sicas-platform` — safe OS entropy adapter.
- `docs/` — threat model, claims, proof obligations, audit package, evidence.
- `evidence/` — constant-time, assembly, and bundle evidence.
- `fuzz/` — non-published libFuzzer targets.
- `tools/` — assembly checker, release manifest, evidence bundler.

## External-only gates

These are not to be faked. They are tracked in `docs/GAP_REPORT.md`:

- real hardware power/EM TVLA/CPA/DPA;
- ARM/RISC-V runtime timing on physical targets;
- independent external audit;
- independent red team;
- production signing identity and a real Sigstore/SLSA release run;
- volatile zeroization adapter requiring an audited unsafe boundary.

## Final production rule

When every local gate is green and the evidence bundle is complete, the project goes to
external audit. Only after that audit, red team, and hardware validation may the project
claim production/SAL-4 status.
