# Working Agreement

This document records the persistent operating agreement for the SiCaS project.

## Autonomy

The implementation agent works locally and autonomously. Routine implementation,
testing, refactoring, documentation, evidence collection, and commit decisions do not
require user confirmation. The agent should choose the highest-value local gap and
continue.

The agent must not ask the user to restate the project goals, scope, quality standard, or
tooling policy. Those are fixed in `AGENTS.md`, `PLAN.md`, and this document.

## Local-first proof

The project is proven locally first. Use every available local tool and custom script:

- `rustfmt`, strict Clippy, rustc lints;
- unit, integration, property, differential, and compile-fail tests;
- Miri, ASan, Kani, fuzzing, mutation testing, coverage;
- multi-architecture assembly checks;
- dudect-style timing evidence;
- TVLA/CPA against synthetic and, when available, hardware traces;
- supply-chain, reproducibility, SBOM, provenance, and release dry runs.

## Evidence discipline

- Every security claim links to evidence in `docs/PROOF_OBLIGATIONS.md`.
- Every evidence run records commit, target, tool version, parameters, and raw result.
- Missing evidence downgrades the claim.
- External or physical gates are recorded in `docs/GAP_REPORT.md`; they are never
  simulated as if they were real.

## Production boundary

Local work can reach a complete audit-ready package. It cannot truthfully complete:

- independent external audit;
- independent red team;
- physical power/EM acquisition;
- production signing identity.

Those happen after local proof is complete. Until then, the label is **SAL-2 locally**
or the highest level actually supported by evidence.

## Commit discipline

- Commit at the end of every phase.
- Keep security-relevant changes isolated from formatting-only changes.
- Never rewrite published history.
- Update `CHANGELOG.md`, `docs/PROOF_OBLIGATIONS.md`, `docs/GAP_REPORT.md`, and the
  evidence bundle when a phase changes claims or evidence.

## Stop conditions

The agent stops only when:

- the local backlog is empty and all local gates are green; or
- a task genuinely requires external hardware, credentials, or an independent party;
- a destructive action would affect user data and no safe alternative exists.

In the second case, the agent records the gap and continues with other local work.
