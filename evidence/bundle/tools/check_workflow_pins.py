#!/usr/bin/env python3
"""Reject GitHub Actions references that are not pinned to a full commit SHA.

A movable tag or branch in a workflow with write/OIDC permissions is a
supply-chain trust boundary. Pinning every third-party action to a 40-character
commit SHA makes the reviewed workflow the thing that actually runs.
"""

from __future__ import annotations

import argparse
import pathlib
import re
import sys

USES = re.compile(r"^\s*-?\s*uses:\s*([^\s#]+)")
SHA = re.compile(r"^[^@]+@[0-9a-f]{40}$")


def check_file(path: pathlib.Path) -> list[str]:
    errors: list[str] = []
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        match = USES.match(line)
        if match is None:
            continue
        reference = match.group(1)
        if reference.startswith("./") or reference.startswith("docker://"):
            continue
        if not SHA.fullmatch(reference):
            errors.append(f"{path}:{number}: action is not pinned to a commit SHA: {reference}")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--directory",
        type=pathlib.Path,
        default=pathlib.Path(".github/workflows"),
    )
    args = parser.parse_args()

    files = sorted(args.directory.glob("*.yml"))
    if not files:
        print(f"no workflow files in {args.directory}", file=sys.stderr)
        return 1

    errors: list[str] = []
    for path in files:
        errors.extend(check_file(path))

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1

    print(f"all {len(files)} workflow file(s) pin actions to commit SHAs")
    return 0


if __name__ == "__main__":
    sys.exit(main())
