#!/usr/bin/env python3
"""Validate release checksums, provenance, and SBOM consistency.

The validator is intentionally strict: a release manifest is accepted only when
the artifact hash, checksum file, provenance record, SBOM component graph, and
(optionally) the current Git commit all agree.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import pathlib
import re
import subprocess
import sys

HEX_DIGEST = re.compile(r"^[0-9a-f]{64}$")
HEX_COMMIT = re.compile(r"^[0-9a-f]{40}$")


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(path: pathlib.Path, errors: list[str]) -> dict | None:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        errors.append(f"cannot read {path.name}: {error}")
        return None
    if not isinstance(value, dict):
        errors.append(f"{path.name} must contain a JSON object")
        return None
    return value


def parse_checksums(path: pathlib.Path, errors: list[str]) -> dict[str, str]:
    entries: dict[str, str] = {}
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as error:
        errors.append(f"cannot read SHA256SUMS: {error}")
        return entries

    for number, line in enumerate(lines, start=1):
        if not line.strip():
            continue
        parts = line.split()
        if len(parts) != 2:
            errors.append(f"SHA256SUMS line {number} is malformed")
            continue
        digest, name = parts
        if not HEX_DIGEST.fullmatch(digest):
            errors.append(f"SHA256SUMS line {number} has an invalid digest")
            continue
        if name in entries:
            errors.append(f"SHA256SUMS lists {name} more than once")
            continue
        entries[name] = digest

    if not entries:
        errors.append("SHA256SUMS contains no artifacts")
    return entries


def parse_timestamp(value: object, field: str, errors: list[str]) -> None:
    if not isinstance(value, str):
        errors.append(f"{field} must be an ISO-8601 string")
        return
    try:
        parsed = dt.datetime.fromisoformat(value)
    except ValueError:
        errors.append(f"{field} is not a valid ISO-8601 timestamp")
        return
    if parsed.tzinfo is None:
        errors.append(f"{field} must include a timezone")


def validate_sbom(sbom: dict, errors: list[str]) -> dict[str, str]:
    if sbom.get("bomFormat") != "CycloneDX":
        errors.append("SBOM bomFormat must be CycloneDX")
    if not isinstance(sbom.get("specVersion"), str):
        errors.append("SBOM specVersion must be a string")
    if not isinstance(sbom.get("version"), int) or sbom.get("version", 0) < 1:
        errors.append("SBOM version must be a positive integer")

    components = sbom.get("components")
    if not isinstance(components, list) or not components:
        errors.append("SBOM components must be a non-empty list")
        return {}

    versions: dict[str, str] = {}
    for component in components:
        if not isinstance(component, dict):
            errors.append("SBOM component must be an object")
            continue
        name = component.get("name")
        version = component.get("version")
        reference = component.get("bom-ref")
        if not isinstance(name, str) or not name:
            errors.append("SBOM component name must be a non-empty string")
            continue
        if not isinstance(version, str) or not version:
            errors.append(f"SBOM component {name} version must be a non-empty string")
            continue
        if not isinstance(reference, str) or not reference:
            errors.append(f"SBOM component {name} bom-ref must be a non-empty string")
            continue
        if reference in versions:
            errors.append(f"SBOM bom-ref {reference} is duplicated")
        versions[reference] = name
        if component.get("purl") != reference:
            errors.append(f"SBOM component {name} purl must match its bom-ref")
        if not component.get("licenses"):
            errors.append(f"SBOM component {name} must declare a license")

    dependencies = sbom.get("dependencies")
    if not isinstance(dependencies, list):
        errors.append("SBOM dependencies must be a list")
        return versions

    referenced: set[str] = set()
    for dependency in dependencies:
        if not isinstance(dependency, dict):
            errors.append("SBOM dependency entry must be an object")
            continue
        reference = dependency.get("ref")
        if not isinstance(reference, str) or reference not in versions:
            errors.append(f"SBOM dependency ref {reference!r} is unknown")
            continue
        referenced.add(reference)
        depends_on = dependency.get("dependsOn", [])
        if not isinstance(depends_on, list):
            errors.append(f"SBOM dependency {reference} dependsOn must be a list")
            continue
        for target in depends_on:
            if not isinstance(target, str) or target not in versions:
                errors.append(f"SBOM dependency {reference} targets unknown {target!r}")

    for reference in versions:
        if reference not in referenced:
            errors.append(f"SBOM component {reference} has no dependency entry")

    metadata = sbom.get("metadata")
    if not isinstance(metadata, dict):
        errors.append("SBOM metadata must be an object")
        return versions
    parse_timestamp(metadata.get("timestamp"), "SBOM metadata.timestamp", errors)

    root = metadata.get("component")
    if not isinstance(root, dict) or root.get("name") != "SiCaS":
        errors.append("SBOM metadata.component must name SiCaS")
    else:
        harness = next(
            (version for reference, name in versions.items() if name == "sicas-harness"),
            None,
        )
        if harness is not None and root.get("version") != harness:
            errors.append("SBOM root version must match sicas-harness")

    return versions


def validate_manifest(
    manifest: pathlib.Path,
    *,
    workspace_packages: dict[str, str] | None = None,
    commit: str | None = None,
) -> list[str]:
    errors: list[str] = []
    required = ["SHA256SUMS", "provenance.json", "sbom.cdx.json", "README.md"]
    for name in required:
        if not (manifest / name).is_file():
            errors.append(f"missing {name}")
    if errors:
        return errors

    checksums = parse_checksums(manifest / "SHA256SUMS", errors)
    provenance = read_json(manifest / "provenance.json", errors)
    sbom = read_json(manifest / "sbom.cdx.json", errors)
    if provenance is None or sbom is None:
        return errors

    artifact_name = provenance.get("artifact")
    if not isinstance(artifact_name, str) or not artifact_name:
        errors.append("provenance.artifact must be a non-empty string")
        artifact_name = None

    if artifact_name is not None:
        if set(checksums) != {artifact_name}:
            errors.append("SHA256SUMS must list exactly the provenance artifact")
        artifact = manifest / artifact_name
        if not artifact.is_file():
            errors.append(f"artifact {artifact_name} is missing")
        else:
            actual = sha256(artifact)
            if checksums.get(artifact_name) != actual:
                errors.append("artifact digest does not match SHA256SUMS")
            if provenance.get("sha256") != actual:
                errors.append("artifact digest does not match provenance.sha256")

    if provenance.get("project") != "SiCaS":
        errors.append("provenance.project must be SiCaS")
    if provenance.get("build_command") != "cargo build --workspace --release --locked":
        errors.append("provenance.build_command is not the canonical release build")
    for field in ("rustc", "host"):
        value = provenance.get(field)
        if not isinstance(value, str) or not value:
            errors.append(f"provenance.{field} must be a non-empty string")
    parse_timestamp(provenance.get("timestamp"), "provenance.timestamp", errors)

    recorded_commit = provenance.get("commit")
    if not isinstance(recorded_commit, str) or not HEX_COMMIT.fullmatch(recorded_commit):
        errors.append("provenance.commit must be a 40-character lowercase hex commit")
    elif commit is not None and recorded_commit != commit:
        errors.append(
            f"provenance.commit {recorded_commit} does not match current commit {commit}"
        )

    versions = validate_sbom(sbom, errors)
    if workspace_packages is not None:
        sbom_versions = {
            component["name"]: component["version"]
            for component in sbom.get("components", [])
            if isinstance(component, dict)
            and isinstance(component.get("name"), str)
            and isinstance(component.get("version"), str)
        }
        for name, version in sorted(workspace_packages.items()):
            if sbom_versions.get(name) != version:
                errors.append(
                    f"SBOM component {name} must be version {version}, "
                    f"found {sbom_versions.get(name)!r}"
                )
        for name in sorted(set(sbom_versions) - set(workspace_packages)):
            errors.append(f"SBOM lists unknown workspace component {name}")

    if artifact_name is not None and provenance.get("sha256") is not None:
        readme = (manifest / "README.md").read_text(encoding="utf-8", errors="replace")
        if artifact_name not in readme:
            errors.append("README.md does not name the artifact")
        if provenance.get("sha256") not in readme:
            errors.append("README.md does not contain the artifact digest")
        if isinstance(recorded_commit, str) and recorded_commit not in readme:
            errors.append("README.md does not contain the commit")

    return errors


def workspace_packages() -> dict[str, str]:
    output = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    ).stdout
    metadata = json.loads(output)
    return {package["name"]: package["version"] for package in metadata["packages"]}


def current_commit() -> str | None:
    try:
        return subprocess.run(
            ["git", "rev-parse", "HEAD"],
            check=True,
            text=True,
            stdout=subprocess.PIPE,
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=pathlib.Path, default=pathlib.Path("release"))
    parser.add_argument("--skip-workspace", action="store_true")
    parser.add_argument("--skip-commit", action="store_true")
    args = parser.parse_args()

    packages = None if args.skip_workspace else workspace_packages()
    commit = None if args.skip_commit else current_commit()
    errors = validate_manifest(args.manifest, workspace_packages=packages, commit=commit)

    if errors:
        for error in errors:
            print(f"release manifest invalid: {error}", file=sys.stderr)
        return 1

    print(f"release manifest OK: {args.manifest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
