# SiCaS Paper Version Log

This log records document revisions and the code/evidence state they describe. The
document version and the code version are separate: the code remains `0.1.0` until the
external gates pass.

| Document version | Date | Code commit | Release build | Evidence tag | Changes |
|---|---|---|---|---|---|
| 0.1.0 | 2026-10-30 | initial tree | — | — | Initial white paper and yellow paper outline: problem statement, threat model, masking model, trace format. |
| 0.9.0 | 2026-10-31 | `8c41e4b` | `8c41e4b` | `local-proof-2026-10-31-final-audit` | Full local proof, evaluation chapter, proof obligations, release format, reproducibility commands. |
| 1.0.0 | 2026-10-31 | `ef13675` | `123eb69` | `final-audit-2026-10-31-dpa` | AES S-box DPA, GPL-3.0-only relicensing, final audit-ready package, coverage/mutation evidence. |

## Versioning policy

- **Major** versions change the specification or the public claim set.
- **Minor** versions add sections, evidence, or algorithms without changing existing
  claims.
- **Patch** versions fix wording, references, or typographical errors.

Every version that changes a security claim must reference the code commit, the release
build commit, the evidence tag, and the proof run that supports the claim. A document
change alone never upgrades the assurance label.
