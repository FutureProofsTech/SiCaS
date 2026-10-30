#!/usr/bin/env python3
"""Collect the reproducible evidence bundle for review or audit."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import pathlib
import shutil
import subprocess
import sys

REQUIRED = [
    "AGENTS.md",
    "opencode.json",
    "PLAN.md",
    "README.md",
    "SECURITY.md",
    "CONTRIBUTING.md",
    "CHANGELOG.md",
    "deny.toml",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "docs/THREAT_MODEL.md",
    "docs/SECURITY_CLAIMS.md",
    "docs/PROOF_OBLIGATIONS.md",
    "docs/ASSURANCE.md",
    "docs/CODE_QUALITY.md",
    "docs/SUPPORT_MATRIX.md",
    "docs/EVALUATION.md",
    "docs/CT_EVIDENCE.md",
    "docs/CROSS_ARCH_EVIDENCE.md",
    "docs/FUZZ_EVIDENCE.md",
    "docs/MUTATION_EVIDENCE.md",
    "docs/HARDWARE.md",
    "docs/RELEASE.md",
    "docs/RED_TEAM.md",
    "docs/TOOLING.md",
    "docs/AUDIT_PACKAGE.md",
    "docs/AUDIT_REPORT_2026-10-30.md",
    "docs/GAP_REPORT.md",
    "docs/LOCAL_PROOF.md",
    "docs/POST_PROOF_BACKLOG.md",
    "docs/WORKING_AGREEMENT.md",
    "docs/ZEROIZATION.md",
    "evidence/ct/2026-10-30-x86_64.md",
    "evidence/ct/asm-2026-10-30.md",
    "evidence/leakage/2026-10-31-x86_64.md",
    "evidence/release/2026-10-30-x86_64.md",
    "evidence/release/2026-10-31-x86_64.md",
    "tools/local_proof.sh",
    "tools/run-aarch64-tests.sh",
    "tools/run-i686-tests.sh",
    "tools/validate_release_manifest.py",
    "tools/test_validate_release_manifest.py",
]

OPTIONAL = [
    "release/SHA256SUMS",
    "release/sbom.cdx.json",
    "release/provenance.json",
]


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def git_commit() -> str:
    try:
        return subprocess.run(
            ["git", "rev-parse", "HEAD"],
            check=True,
            text=True,
            stdout=subprocess.PIPE,
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return "unknown"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=pathlib.Path, default=pathlib.Path("evidence/bundle"))
    args = parser.parse_args()

    root = pathlib.Path.cwd()
    output = args.output
    if output.exists():
        shutil.rmtree(output)
    output.mkdir(parents=True)

    missing = [name for name in REQUIRED if not (root / name).is_file()]
    if missing:
        for name in missing:
            print(f"missing required evidence file: {name}", file=sys.stderr)
        return 1

    files = list(REQUIRED)
    files.extend(name for name in OPTIONAL if (root / name).is_file())

    rows = []
    for name in files:
        source = root / name
        destination = output / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
        rows.append((name, sha256(destination)))

    manifest = "\n".join(f"{digest}  {name}" for name, digest in rows) + "\n"
    (output / "MANIFEST.sha256").write_text(manifest, encoding="utf-8")

    index = [
        "# SiCaS Evidence Bundle",
        "",
        f"- Commit: `{git_commit()}`",
        f"- Generated: `{dt.datetime.now(dt.timezone.utc).isoformat()}`",
        f"- Files: {len(rows)}",
        "",
        "| File | SHA-256 |",
        "|---|---|",
    ]
    index.extend(f"| `{name}` | `{digest}` |" for name, digest in rows)
    (output / "INDEX.md").write_text("\n".join(index) + "\n", encoding="utf-8")

    print(f"wrote {output} with {len(rows)} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
