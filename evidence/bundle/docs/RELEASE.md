# Release Process

## Release gates

A release candidate must pass:

- format, strict Clippy, unit/integration/doc tests;
- Miri, ASan, and Kani;
- fuzz campaigns for all targets;
- mutation testing with no missed mutants in core security logic;
- coverage at or above 90% lines;
- `cargo audit` and `cargo deny check`;
- reproducible build comparison;
- multi-architecture assembly checks;
- independent audit and red-team sign-off before a production claim.

## Build artifacts

```bash
python3 tools/release_manifest.py --output release
```

The script builds the workspace, copies `sicas-harness`, and writes:

- `release/sicas-harness`
- `release/SHA256SUMS`
- `release/sbom.cdx.json`
- `release/provenance.json`
- `release/README.md`

Verify the artifact and the manifest consistency:

```bash
cd release
sha256sum -c SHA256SUMS
cd ..
python3 tools/validate_release_manifest.py --manifest release
```

The validator checks the artifact hash, provenance, SBOM component graph, workspace
versions, timestamps, and the current commit. It passes in the full local proof.

## Local signing dry run

The checksum-signing path can be exercised locally with an ephemeral test key:

```bash
tools/local-signing-dry-run.sh release evidence/signing
```

This proves that a detached signature can be produced and verified. The test key is
temporary and is not a production signing identity; the production signing and SLSA
gates remain open.

## Signing

Signing keys are not stored in this repository. Before a production release:

1. Generate or load an offline release key.
2. Sign `SHA256SUMS` with minisign or Sigstore.
3. Publish the signature and public verification key alongside the artifact.
4. Verify the signature in a clean environment.
5. Record the signing identity and timestamp in the release evidence bundle.

The current manifest script produces unsigned artifacts; signing remains an
explicit release gate. `.github/workflows/release.yml` builds the manifest, attests
provenance, signs the artifact with Sigstore keyless signing, and creates a draft
GitHub release. It requires a release tag and GitHub OIDC permissions.

## Supply chain

- `deny.toml` enforces license, advisory, source, wildcard, and duplicate-version
  policy.
- `Cargo.lock` is committed.
- The release profile uses `opt-level=z`, fat LTO, one codegen unit, `panic=abort`, and
stripped symbols. The measured release binary is 386,232 bytes and the masked-ChaCha20
batch is 7.15× vs unmasked in the final full proof (7.26× in an earlier run). The build uses a pinned toolchain.
- The SBOM lists every workspace component and dependency edge.
- SLSA provenance is planned but not yet implemented.

## Disclosure and incident response

- `SECURITY.md` defines reporting and disclosure.
- A release must have a named security contact and a CVE response path.
- Revoked or compromised artifacts must be withdrawn and re-signed.

## Current status

The build and manifest path is implemented and reproducible locally. Signing,
SLSA provenance, and an independent release review are still outstanding.
