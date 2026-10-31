# Red Team Program

## Independence

The red team must be independent of the implementation team. Internal adversarial
review is useful but is not sufficient for a production claim.

## Scope

In scope:

- `sicas-core`, `sicas-harness`, `sicas-reference`, and `sicas-platform`;
- the CLI, trace parser, synthetic generator, and timing suite;
- build, CI, release, and artifact-integrity paths;
- the documented threat model and security claims.

Out of scope:

- third-party OS, toolchain, and hardware vulnerabilities;
- real user data or production systems;
- destructive physical attacks;
- social engineering.

Fault injection and physical probing are in scope only on dedicated lab hardware.

## Rules of engagement

- Written authorization and scope document before testing.
- Isolated lab and synthetic keys only.
- Defined stop conditions and emergency contact.
- Secure handling of traces, PoCs, and findings.
- No testing outside the agreed window.
- Safe-harbor terms and coordinated disclosure.

## Attack catalog

| Category | Tests |
|---|---|
| Side-channel | SPA, DPA, CPA, template, higher-order, power/EM, alignment, multiple devices, temperature/voltage |
| Masking bypass | Compiler optimization, LTO, share recombination, glitches, declassification points, RNG reuse/bias |
| Timing | dudect, cache/branch/memory-access timing, early exits, variable-latency paths |
| Fault injection | Voltage/clock glitching, RNG stuck-at/bias, entropy starvation, truncated input, panic/unwind |
| API misuse | Secret in logs/errors/`Debug`, clone/equality/index misuse, zeroization gaps, declassification abuse |
| Build/supply chain | Reproducibility mismatch, artifact substitution, CI compromise, toolchain tampering, signed-tag verification |
| Parser/DoS | Huge inputs, integer overflow, allocation failure, hangs, CLI fuzzing |
| Protocol/usage | Nonce reuse, key lifetime, error oracles, unsafe configuration |
| Detection/response | Can the team detect, triage, remediate, and retest? |

## Reviewer commands

```bash
tools/auditor_smoke.sh /tmp/sicas-auditor-smoke
cargo test -p sicas-reference bitsliced
cargo +nightly-2026-08-03 fuzz run chacha20 -- -runs=1000000
cargo mutants -p sicas-core -p sicas-reference --no-shuffle
```

The smoke test covers core masking, bitsliced ChaCha20, leakage model, statistics,
assembly matrix, and short timing. Use `AUDITOR_GUIDE.md` for the full path.

## Severity and remediation

| Severity | Examples | SLA | Release impact |
|---|---|---|---|
| Critical | Key recovery, fail-open entropy, code execution | 7 days | Blocks release |
| High | Practical masking bypass, secret disclosure, build compromise | 30 days | Blocks release |
| Medium | Limited leakage, DoS, defense-in-depth gap | 90 days | Risk acceptance |
| Low | Hardening, documentation, minor leak | Next release | Tracked |

Every finding requires a reproducible PoC, an owner, a fix, and a retest.

## Deliverables

- Scope and authorization record.
- Red-team report with reproducible PoCs and traces.
- Remediation tickets and retest report.
- Public summary and CVE identifiers where appropriate.
- Coordinated disclosure timeline and credit policy.

## Current status

No independent red-team engagement has been performed. This document defines the
program; it is not evidence of a completed review.
