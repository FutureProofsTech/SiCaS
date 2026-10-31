# Production Readiness

## Status

- Code commit: `ef13675`
- Release build commit: `123eb69`
- Evidence tag: `final-audit-2026-10-31-dpa`
- Public label: **SAL-2 locally**
- Production status: **not certified**
- Deployment rule: **do not use to protect real secrets until the independent and physical gates below are complete**

The local engineering proof is complete. This document separates “ready to review and
release” from “certified for production secrets.”

## What is production-ready locally

- Safe Rust, `#![forbid(unsafe_code)]` in published crates.
- Zero third-party runtime dependencies.
- Strict formatting, Clippy, restriction lints, and rustdoc.
- Unit, integration, property, differential, and compile-fail tests.
- Miri, ASan, Valgrind, Kani, mutation, coverage, and five 1M-execution fuzz campaigns.
- 69-configuration constant-time assembly matrix, including `opt-level=z` and fat LTO.
- Bitsliced masked ChaCha20 with a first-order masking argument and lane tests.
- Reproducible release builds, SBOM, provenance, checksums, and manifest validation.
- Local signing and verification dry run with an ephemeral test key.
- Auditor guide, checklist, technique matrix, and hashed evidence bundle.

## What blocks production certification

| Gate | Status | Why it is not local |
|---|---|---|
| Independent security audit | Pending | Requires an independent auditor |
| Independent red team | Pending | Requires an independent team |
| Hardware power/EM TVLA/CPA/DPA | Pending | Requires a physical target and acquisition lab |
| Production signing identity / SLSA | Pending | Requires a trusted release identity |
| Volatile zeroization | Experimental adapter implemented | `sicas-zeroize` is isolated and unaudited; it is not used by published crates |
| 5× performance target | Open | 10× hard ceiling is met; 5× needs further optimization |

## Release commands

```bash
./tools/local_proof.sh full
python3 tools/release_manifest.py --output release
python3 tools/validate_release_manifest.py --manifest release
# Add --skip-commit when validating from the evidence tag: the manifest records
# build commit 123eb69 (proof code ef13675), while the tag includes later
# docs/evidence commits.
tools/local-signing-dry-run.sh release evidence/signing
```

The release binary SHA-256 for the reviewed commit is:

```text
1f0d304ca9b9582fd9153e7c006c240bd0e06acc3a685dfad93088429e4de7da
```

Size: 386,232 bytes.

## Audit handoff

Give the reviewer:

1. `AUDITOR_GUIDE.md`
2. `docs/AUDITOR_CHECKLIST.md`
3. `docs/TEST_MATRIX.md`
4. `docs/AUDIT_TECHNIQUE_MATRIX.md`
5. `docs/AUDIT_PACKAGE.md`
6. `evidence/bundle/`
7. the proof code at `ef13675` and the release build commit `123eb69`

The reviewer can verify the bundle with:

```bash
cd evidence/bundle
sha256sum -c MANIFEST.sha256
```

## Production decision

The project is ready for independent review and release engineering. It is not ready
for a production security claim. That claim requires the pending independent and
physical evidence, and it must not be inferred from the local proof.
