#!/usr/bin/env python3
"""Tests for the release-manifest validator."""

from __future__ import annotations

import hashlib
import json
import pathlib
import tempfile
import unittest

import validate_release_manifest as validator

COMMIT = "a" * 40
TIMESTAMP = "2026-01-01T00:00:00+00:00"


def build_manifest(root: pathlib.Path) -> tuple[pathlib.Path, dict[str, str]]:
    manifest = root / "release"
    manifest.mkdir()
    artifact = b"release-binary"
    digest = hashlib.sha256(artifact).hexdigest()
    (manifest / "sicas-harness").write_bytes(artifact)
    (manifest / "SHA256SUMS").write_text(
        f"{digest}  sicas-harness\n", encoding="utf-8"
    )
    (manifest / "provenance.json").write_text(
        json.dumps(
            {
                "project": "SiCaS",
                "commit": COMMIT,
                "rustc": "rustc 1.99.0",
                "host": "x86_64-unknown-linux-gnu",
                "build_command": "cargo build --workspace --release --locked",
                "artifact": "sicas-harness",
                "sha256": digest,
                "timestamp": TIMESTAMP,
            }
        ),
        encoding="utf-8",
    )
    components = [
        {
            "type": "library",
            "bom-ref": "pkg:cargo/sicas-harness@0.1.0",
            "name": "sicas-harness",
            "version": "0.1.0",
            "licenses": [{"license": {"name": "GPL-3.0-only"}}],
            "purl": "pkg:cargo/sicas-harness@0.1.0",
        },
        {
            "type": "library",
            "bom-ref": "pkg:cargo/sicas-core@0.1.0",
            "name": "sicas-core",
            "version": "0.1.0",
            "licenses": [{"license": {"name": "GPL-3.0-only"}}],
            "purl": "pkg:cargo/sicas-core@0.1.0",
        },
    ]
    (manifest / "sbom.cdx.json").write_text(
        json.dumps(
            {
                "bomFormat": "CycloneDX",
                "specVersion": "1.5",
                "version": 1,
                "metadata": {
                    "timestamp": TIMESTAMP,
                    "component": {"type": "application", "name": "SiCaS", "version": "0.1.0"},
                },
                "components": components,
                "dependencies": [
                    {
                        "ref": "pkg:cargo/sicas-harness@0.1.0",
                        "dependsOn": ["pkg:cargo/sicas-core@0.1.0"],
                    },
                    {"ref": "pkg:cargo/sicas-core@0.1.0", "dependsOn": []},
                ],
            }
        ),
        encoding="utf-8",
    )
    (manifest / "README.md").write_text(
        f"# SiCaS release\n\n- Commit: `{COMMIT}`\n- Artifact: `sicas-harness`\n"
        f"- SHA-256: `{digest}`\n",
        encoding="utf-8",
    )
    return manifest, {"sicas-harness": "0.1.0", "sicas-core": "0.1.0"}


class ReleaseManifestTests(unittest.TestCase):
    def test_valid_manifest_passes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest, packages = build_manifest(pathlib.Path(directory))
            errors = validator.validate_manifest(
                manifest, workspace_packages=packages, commit=COMMIT
            )
            self.assertEqual(errors, [])

    def test_tampered_artifact_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest, packages = build_manifest(pathlib.Path(directory))
            (manifest / "sicas-harness").write_bytes(b"tampered")
            errors = validator.validate_manifest(
                manifest, workspace_packages=packages, commit=COMMIT
            )
            self.assertTrue(any("SHA256SUMS" in error for error in errors))

    def test_wrong_commit_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest, packages = build_manifest(pathlib.Path(directory))
            errors = validator.validate_manifest(
                manifest, workspace_packages=packages, commit="b" * 40
            )
            self.assertTrue(any("does not match current commit" in error for error in errors))

    def test_missing_dependency_entry_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest, packages = build_manifest(pathlib.Path(directory))
            sbom = json.loads((manifest / "sbom.cdx.json").read_text(encoding="utf-8"))
            sbom["dependencies"] = sbom["dependencies"][:1]
            (manifest / "sbom.cdx.json").write_text(json.dumps(sbom), encoding="utf-8")
            errors = validator.validate_manifest(
                manifest, workspace_packages=packages, commit=COMMIT
            )
            self.assertTrue(any("no dependency entry" in error for error in errors))

    def test_workspace_version_mismatch_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest, packages = build_manifest(pathlib.Path(directory))
            packages["sicas-core"] = "9.9.9"
            errors = validator.validate_manifest(
                manifest, workspace_packages=packages, commit=COMMIT
            )
            self.assertTrue(any("sicas-core" in error for error in errors))

    def test_naive_timestamp_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest, packages = build_manifest(pathlib.Path(directory))
            provenance = json.loads(
                (manifest / "provenance.json").read_text(encoding="utf-8")
            )
            provenance["timestamp"] = "2026-01-01T00:00:00"
            (manifest / "provenance.json").write_text(
                json.dumps(provenance), encoding="utf-8"
            )
            errors = validator.validate_manifest(
                manifest, workspace_packages=packages, commit=COMMIT
            )
            self.assertTrue(any("timezone" in error for error in errors))

    def test_duplicate_component_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest, packages = build_manifest(pathlib.Path(directory))
            sbom = json.loads((manifest / "sbom.cdx.json").read_text(encoding="utf-8"))
            sbom["components"].append(dict(sbom["components"][0]))
            (manifest / "sbom.cdx.json").write_text(json.dumps(sbom), encoding="utf-8")
            errors = validator.validate_manifest(
                manifest, workspace_packages=packages, commit=COMMIT
            )
            self.assertTrue(any("duplicated" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
