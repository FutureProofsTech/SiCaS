#!/usr/bin/env python3
"""Build release artifacts and emit checksums, SBOM, and provenance metadata."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys


def run(command: list[str]) -> str:
    return subprocess.run(
        command,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    ).stdout.strip()


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def workspace_metadata() -> dict:
    return json.loads(run(["cargo", "metadata", "--format-version", "1", "--no-deps"]))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=pathlib.Path, default=pathlib.Path("release"))
    parser.add_argument("--skip-build", action="store_true")
    args = parser.parse_args()

    if not args.skip_build:
        run(["cargo", "build", "--workspace", "--release", "--locked"])

    metadata = workspace_metadata()
    members = {package["name"] for package in metadata["packages"]}
    root = pathlib.Path(metadata["workspace_root"])
    output = args.output
    output.mkdir(parents=True, exist_ok=True)

    binary_name = "sicas-harness.exe" if os.name == "nt" else "sicas-harness"
    binary = root / "target" / "release" / binary_name
    if not binary.is_file():
        raise SystemExit(f"release binary not found: {binary}")
    artifact = output / binary_name
    shutil.copy2(binary, artifact)

    digest = sha256(artifact)
    (output / "SHA256SUMS").write_text(
        f"{digest}  {binary_name}\n", encoding="utf-8"
    )

    rustc_version = run(["rustc", "--version"])
    rustc_verbose = run(["rustc", "-vV"])
    host = next(
        (line.split(":", 1)[1].strip() for line in rustc_verbose.splitlines() if line.startswith("host:")),
        "unknown",
    )
    try:
        commit = run(["git", "rev-parse", "HEAD"])
    except (OSError, subprocess.CalledProcessError):
        commit = "unknown"

    timestamp = dt.datetime.now(dt.timezone.utc).isoformat()
    provenance = {
        "project": "SiCaS",
        "commit": commit,
        "rustc": rustc_version,
        "host": host,
        "build_command": "cargo build --workspace --release --locked",
        "artifact": binary_name,
        "sha256": digest,
        "timestamp": timestamp,
    }
    (output / "provenance.json").write_text(
        json.dumps(provenance, indent=2) + "\n", encoding="utf-8"
    )

    components = []
    dependencies = []
    for package in metadata["packages"]:
        name = package["name"]
        version = package["version"]
        reference = f"pkg:cargo/{name}@{version}"
        components.append(
            {
                "type": "library",
                "bom-ref": reference,
                "name": name,
                "version": version,
                "licenses": [
                    {"license": {"name": package.get("license") or "NOASSERTION"}}
                ],
                "purl": reference,
            }
        )
        depends_on = sorted(
            f"pkg:cargo/{dependency['name']}@{next(p['version'] for p in metadata['packages'] if p['name'] == dependency['name'])}"
            for dependency in package["dependencies"]
            if dependency["name"] in members
        )
        dependencies.append({"ref": reference, "dependsOn": depends_on})

    sbom = {
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "version": 1,
        "metadata": {
            "timestamp": timestamp,
            "tools": [
                {
                    "vendor": "SiCaS",
                    "name": "release_manifest",
                    "version": "0.1.0",
                }
            ],
            "component": {
                "type": "application",
                "name": "SiCaS",
                "version": next(
                    package["version"]
                    for package in metadata["packages"]
                    if package["name"] == "sicas-harness"
                ),
            },
        },
        "components": components,
        "dependencies": dependencies,
    }
    (output / "sbom.cdx.json").write_text(
        json.dumps(sbom, indent=2) + "\n", encoding="utf-8"
    )

    (output / "README.md").write_text(
        "# SiCaS release artifacts\n\n"
        f"- Commit: `{commit}`\n"
        f"- Host: `{host}`\n"
        f"- Rust: `{rustc_version}`\n"
        f"- Artifact: `{binary_name}`\n"
        f"- SHA-256: `{digest}`\n\n"
        "Verify with `sha256sum -c SHA256SUMS`.\n"
        "Signing is a separate step; see `docs/RELEASE.md`.\n",
        encoding="utf-8",
    )

    print(f"wrote {artifact}")
    print(f"sha256 {digest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
