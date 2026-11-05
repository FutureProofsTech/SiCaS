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

REQUIRED_GLOBS = [
    "crates/**/*.rs",
    "crates/**/Cargo.toml",
    "fuzz/fuzz_targets/*.rs",
    "fuzz/corpus/*/seed",
    "tools/*.py",
    "tools/*.sh",
    ".github/workflows/*.yml",
    "docs/**/*.md",
]

REQUIRED = [
    "AUDITOR_GUIDE.md",
    "AUDIT_REQUEST.md",
    "PRODUCTION_READINESS.md",
    "PLAN.md",
    "README.md",
    "LICENSE",
    "papers/README.md",
    "papers/VERSIONS.md",
    "papers/references.bib",
    "papers/whitepaper/SiCaS-Whitepaper.tex",
    "papers/whitepaper/SiCaS-Whitepaper.pdf",
    "papers/yellowpaper/SiCaS-Yellowpaper.tex",
    "papers/yellowpaper/SiCaS-Yellowpaper.pdf",
    "SECURITY.md",
    "CONTRIBUTING.md",
    "CHANGELOG.md",
    "deny.toml",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "rustfmt.toml",
    "fuzz/Cargo.toml",
    "fuzz/Cargo.lock",
    "docs/THREAT_MODEL.md",
    "docs/SECURITY_CLAIMS.md",
    "docs/PROOF_OBLIGATIONS.md",
    "docs/ASSURANCE.md",
    "docs/BITSLICED_MASKING.md",
    "docs/CODE_QUALITY.md",
    "docs/SUPPORT_MATRIX.md",
    "docs/EVALUATION.md",
    "docs/CT_EVIDENCE.md",
    "docs/CROSS_ARCH_EVIDENCE.md",
    "docs/FUZZ_EVIDENCE.md",
    "docs/MUTATION_EVIDENCE.md",
    "docs/PERFORMANCE_EVIDENCE.md",
    "docs/HARDWARE.md",
    "docs/RELEASE.md",
    "docs/RED_TEAM.md",
    "docs/TOOLING.md",
    "docs/AUDIT_PACKAGE.md",
    "docs/AUDITOR_CHECKLIST.md",
    "docs/AUDIT_TECHNIQUE_MATRIX.md",
    "docs/TEST_MATRIX.md",
    "docs/AUDIT_REPORT_2026-10-30.md",
    "docs/SELF_AUDIT_2026-10-31.md",
    "docs/GAP_REPORT.md",
    "docs/LOCAL_PROOF.md",
    "docs/POST_PROOF_BACKLOG.md",
    "docs/SAL4_LOCAL_READINESS.md",
    "docs/SAL4_EXECUTION_PLAN.md",
    "docs/WORKING_AGREEMENT.md",
    "docs/ZEROIZATION.md",
    "evidence/ct/2026-10-30-x86_64.md",
    "evidence/ct/asm-2026-10-30.md",
    "evidence/ct/asm-2026-10-31.md",
    "evidence/auditor-smoke/2026-10-31.md",
    "evidence/ct/2026-10-31-x86_64-release.md",
    "evidence/leakage/2026-10-31-x86_64.md",
    "evidence/local-proof-2026-10-31/SUMMARY.md",
    "evidence/local-proof-2026-10-31-final/SUMMARY.md",
    "evidence/local-proof-2026-10-31-bitsliced/SUMMARY.md",
    "evidence/local-proof-2026-10-31-speed-size/SUMMARY.md",
    "evidence/local-proof-2026-10-31-self-audit/SUMMARY.md",
    "evidence/local-proof-2026-10-31-self-audit-final/SUMMARY.md",
    "evidence/local-proof-2026-10-31-final-audit/SUMMARY.md",
    "evidence/local-proof-2026-10-31-final-quick/SUMMARY.md",
    "evidence/local-proof-2026-10-31-dpa/SUMMARY.md",
    "evidence/local-proof-2026-10-31-sbox-dpa/SUMMARY.md",
    "evidence/local-proof-2026-10-31-ci-fixes/SUMMARY.md",
    "evidence/local-proof-20261105-final/SUMMARY.md",
    "evidence/local-proof-20261105-final/tests.log",
    "evidence/local-proof-20261105-final/assembly.md",
    "evidence/local-proof-20261105-final/stats.md",
    "evidence/local-proof-20261105-final/workflow-pins.log",
    "evidence/local-proof-20261105-final/workflow-pin-tests.log",
    "evidence/local-proof-20261105-final/valgrind/SUMMARY.md",
    "evidence/local-proof-20261105-ci-fix/SUMMARY.md",
    "evidence/local-proof-20261105-ci-fix/tests.log",
    "evidence/local-proof-20261105-ci-fix/assembly.md",
    "evidence/local-proof-20261105-ci-fix/reference-ct.log",
    "evidence/local-proof-20261105-ci-fix/reference-ct-tests.log",
    "evidence/local-proof-20261105-ci-fix/stats.md",
    "evidence/local-proof-20261105-ci-fix/workflow-pins.log",
    "evidence/local-proof-20261105-ci-fix/workflow-pin-tests.log",
    "evidence/local-proof-20261105-ci-fix/valgrind/SUMMARY.md",
    "evidence/local-proof-20261105-streaming/SUMMARY.md",
    "evidence/local-proof-20261105-streaming/reference-ct.md",
    "evidence/local-proof-20261105-streaming/reference-ct.log",
    "evidence/local-proof-20261105-streaming/reference-ct-tests.log",
    "evidence/local-proof-20261105-streaming/tests.log",
    "evidence/local-proof-20261105-streaming/assembly.md",
    "evidence/local-proof-20261105-streaming/stats.md",
    "evidence/local-proof-20261105-streaming/workflow-pins.log",
    "evidence/local-proof-20261105-streaming/workflow-pin-tests.log",
    "evidence/local-proof-20261105-streaming/valgrind/SUMMARY.md",
    "evidence/local-proof-20261105-full/SUMMARY.md",
    "evidence/local-proof-20261105-full/performance.log",
    "evidence/local-proof-20261105-full/coverage.log",
    "evidence/local-proof-20261105-full/mutation-core.log",
    "evidence/local-proof-20261105-full/mutation-reference.log",
    "evidence/local-proof-20261105-full/mutation-workspace.log",
    "evidence/local-proof-20261105-full/fuzz-masked.log",
    "evidence/local-proof-20261105-full/fuzz-lookup.log",
    "evidence/local-proof-20261105-full/fuzz-trace.log",
    "evidence/local-proof-20261105-full/fuzz-chacha.log",
    "evidence/local-proof-20261105-full/fuzz-chacha-batch.log",
    "evidence/local-proof-20261105-full/aarch64.log",
    "evidence/local-proof-20261105-full/i686.log",
    "evidence/local-proof-20261105-full/release-validate.log",
    "evidence/local-proof-20261105-full/signing-dry-run.log",
    "evidence/local-proof-20261105-full/release/SHA256SUMS",
    "evidence/local-proof-20261105-full/release/provenance.json",
    "evidence/local-proof-20261105-full/release/sbom.cdx.json",
    "evidence/local-proof-20261105-full/release/README.md",
    "evidence/local-proof-20261105-full/signing/SUMMARY.md",
    "evidence/release/2026-10-30-x86_64.md",
    "evidence/signing/2026-10-31-x86_64.md",
    "evidence/signing/2026-10-31-sbox-dpa/SUMMARY.md",
    "evidence/fuzz-long/SUMMARY.md",
    "evidence/fuzz-long/masked_ops.log",
    "evidence/fuzz-long/ct_lookup.log",
    "evidence/fuzz-long/trace_parser.log",
    "evidence/fuzz-long/chacha20.log",
    "evidence/stats/2026-10-31-x86_64.md",
    "evidence/valgrind/2026-10-31-x86_64.md",
    "evidence/mutation/2026-10-31-workspace.md",
    "evidence/coverage/2026-10-31-dpa.md",
    "evidence/coverage/2026-10-31-dpa.log",
    "evidence/mutation/2026-10-31-workspace.log",
    "evidence/mutation/2026-10-31-dpa.log",
    "evidence/zeroize/2026-10-31-x86_64.md",
    "evidence/release/2026-10-31-x86_64.md",
    "evidence/release/2026-10-31-sbox-dpa/README.md",
    "tools/local_proof.sh",
    "tools/auditor_smoke.sh",
    "fuzz/Cargo.toml",
    "fuzz/fuzz_targets/chacha20_batch.rs",
    "fuzz/corpus/chacha20_batch/seed",
    "tools/run-aarch64-tests.sh",
    "tools/run-i686-tests.sh",
    "tools/ct_asm_check.py",
    "tools/check_stats.py",
    "tools/run-valgrind.sh",
    "tools/run-fuzz-long.sh",
    "tools/local-signing-dry-run.sh",
    "tools/run-release-timing.sh",
    "tools/release_manifest.py",
    "tools/evidence_bundle.py",
    "tools/validate_release_manifest.py",
    "tools/test_validate_release_manifest.py",
    "crates/sicas-core/examples/ct_asm.rs",
    "crates/sicas-harness/examples/stats_fixture.rs",
    "crates/sicas-harness/src/dpa.rs",
    "crates/sicas-reference/examples/benchmark.rs",
    "crates/sicas-zeroize/Cargo.toml",
    "crates/sicas-zeroize/src/lib.rs",
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


def git_output(arguments: list[str]) -> str | None:
    try:
        return subprocess.run(
            ["git", *arguments],
            check=True,
            text=True,
            stdout=subprocess.PIPE,
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return None


def git_commit() -> str:
    return git_output(["rev-parse", "HEAD"]) or "unknown"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=pathlib.Path, default=pathlib.Path("evidence/bundle"))
    parser.add_argument(
        "--commit",
        default=None,
        help="code commit under test; defaults to the current Git HEAD",
    )
    parser.add_argument(
        "--allow-dirty",
        action="store_true",
        help="allow a bundle from a working tree with tracked modifications",
    )
    parser.add_argument(
        "--allow-different-commit",
        action="store_true",
        help="allow --commit to differ from the current Git HEAD",
    )
    args = parser.parse_args()

    root = pathlib.Path.cwd()
    head = git_commit()
    status = git_output(["status", "--porcelain", "--untracked-files=no"])
    dirty = bool(status)

    if dirty and not args.allow_dirty:
        print(
            "evidence bundle: tracked files are modified; commit or use --allow-dirty",
            file=sys.stderr,
        )
        return 1

    if args.commit is not None and head != "unknown" and args.commit != head:
        if not args.allow_different_commit:
            print(
                f"evidence bundle: --commit {args.commit} does not match HEAD {head}",
                file=sys.stderr,
            )
            return 1

    output = args.output
    root_resolved = root.resolve()
    output_resolved = output.resolve()
    if output_resolved == root_resolved or output_resolved in root_resolved.parents:
        print(
            f"evidence bundle: refusing to replace a directory that contains the repository: {output}",
            file=sys.stderr,
        )
        return 1

    missing = [name for name in REQUIRED if not (root / name).is_file()]
    for pattern in REQUIRED_GLOBS:
        if not any(path.is_file() for path in root.glob(pattern)):
            missing.append(f"glob {pattern}")
    if missing:
        for name in missing:
            print(f"missing required evidence file: {name}", file=sys.stderr)
        return 1

    if output.exists():
        shutil.rmtree(output)
    output.mkdir(parents=True)

    files = set(REQUIRED)
    for pattern in REQUIRED_GLOBS:
        files.update(
            str(path.relative_to(root)) for path in root.glob(pattern) if path.is_file()
        )
    files.update(name for name in OPTIONAL if (root / name).is_file())
    files = sorted(files)

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
        f"- Code commit under test: `{args.commit or head}`",
        f"- Generated: `{dt.datetime.now(dt.timezone.utc).isoformat()}`",
        f"- Working tree dirty (tracked files): `{dirty}`",
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
