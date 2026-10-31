# SAL-4 Execution Plan

SAL-4 is the target. This plan tracks every criterion, the evidence required, and what
is locally complete versus pending independent execution. It is not a SAL-4 claim.

For the release/deployment view, see `PRODUCTION_READINESS.md`. For the formal handoff,
see `AUDIT_REQUEST.md`.

## Criteria

| SAL-4 criterion | Required evidence | Local status | Next execution |
|---|---|---|---|
| SAL-2 baseline | tests, Miri, sanitizers, fuzzing, static analysis | Complete | Re-run in final proof |
| Formal/model checking | Kani proofs for critical invariants | Complete: 30 harnesses | Extend if new invariants appear |
| Real-hardware TVLA/CPA/DPA | raw power/EM traces, target details, analysis | Pending physical hardware | Execute `docs/HARDWARE.md` on the target |
| Independent security audit | signed audit report, findings, retest | Pending independent auditor | Provide `AUDITOR_GUIDE.md` and evidence bundle |
| Independent red team | scoped report, PoCs, remediation, retest | Pending independent team | Provide `docs/RED_TEAM.md` and attack catalog |
| Reproducible builds | independent build hashes | Complete locally | Re-run on release commit |
| Signed release | trusted signing identity, signature, verification | Local dry run only | Run the release workflow with a production identity |
| No open critical/high findings | audit + red-team closure | No open local critical/high; independent findings pending | Close external findings and retest |
| Public evidence bundle | hashed bundle, claims, gaps, raw data | Complete locally | Publish after independent results |
| Volatile zeroization | audited unsafe boundary or platform guarantee | Experimental `sicas-zeroize` adapter implemented; unaudited | Independent review of the unsafe boundary |

## Local completion rule

A local criterion is complete only when the command, raw result, commit, and tool
version are in the evidence bundle. A criterion that requires a physical target or an
independent party is marked **pending independent execution**, not skipped and not
simulated.

## Current boundary

The local package already contains the SAL-3 prerequisites for model checking and
reproducibility. SAL-3 and SAL-4 still require physical leakage measurements and
independent review. Those are the only criteria not executable on this PC.
