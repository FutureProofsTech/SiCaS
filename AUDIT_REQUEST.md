# Independent Audit Request

## Project

SiCaS — side-channel attenuation and leakage-assessment software.

## Review target

- Code commit: `ef13675`
- Release build commit: `123eb69`
- Evidence tag: `final-audit-2026-10-31-dpa`
- Release SHA-256: `1f0d304ca9b9582fd9153e7c006c240bd0e06acc3a685dfad93088429e4de7da`
- Public label at request time: SAL-2 locally

## Requested scope

- `sicas-core`: constant-time primitives, two-share Boolean masking, masked addition.
- `sicas-reference`: scalar and 64-block bitsliced masked ChaCha20.
- `sicas-harness`: trace parser, preprocessing, TVLA, CPA, timing, leakage model, CLI.
- `sicas-platform`: OS entropy adapter.
- Build, CI, release, SBOM, provenance, and evidence bundle.
- Documentation claims and non-claims.

## Questions

1. Is the two-share masked adder correct and first-order secure under the stated
   probing model?
2. Does the bitsliced batch compose addition, XOR, and rotation without introducing
   first-order leakage or lane cross-talk?
3. Does every masked AND use fresh, independent randomness, and is the initial state
   independently refreshed?
4. Are the constant-time claims supported by the 69-configuration assembly matrix and
   dudect evidence?
5. Can entropy failure leave the scalar and batch state inconsistent?
6. Can the parser, CLI, or synthetic generator be made to panic, hang, or exhaust
   memory?
7. Are the release artifacts reproducible, traceable, and correctly described by the
   SBOM and provenance?
8. Are the documented claims narrower than or equal to the evidence?

## Materials

- `AUDITOR_GUIDE.md`
- `docs/AUDITOR_CHECKLIST.md`
- `docs/TEST_MATRIX.md`
- `docs/AUDIT_TECHNIQUE_MATRIX.md`
- `docs/AUDIT_PACKAGE.md`
- `docs/THREAT_MODEL.md`
- `docs/SECURITY_CLAIMS.md`
- `docs/PROOF_OBLIGATIONS.md`
- `docs/GAP_REPORT.md`
- `evidence/bundle/`

## Rules of engagement

- Use synthetic keys and data only.
- Do not test production systems or real user data.
- Use isolated systems and dedicated lab hardware where applicable.
- Provide a written scope and authorization record.
- Report findings privately through the channel in `SECURITY.md`.
- Include reproducible PoCs and affected claims.

## Expected deliverables

- Audit report with severity, affected claim, reproduction, and retest status.
- Red-team report with PoCs and traces where applicable.
- Hardware TVLA/CPA/DPA report with raw traces and acquisition parameters.
- Remediation and retest record.
- Statement of which SAL-4 criteria are independently verified.
