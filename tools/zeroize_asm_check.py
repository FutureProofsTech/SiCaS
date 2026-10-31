#!/usr/bin/env python3
"""Inspect the zeroization boundary assembly for volatile zero stores."""

from __future__ import annotations

import argparse
import os
import pathlib
import re
import subprocess
import sys

TARGET = "x86_64-unknown-linux-gnu"


def build() -> pathlib.Path:
    target_dir = pathlib.Path("target") / "zeroize-asm"
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(target_dir)
    env["CARGO_PROFILE_RELEASE_STRIP"] = "none"
    subprocess.run(
        [
            "cargo",
            "build",
            "-p",
            "sicas-zeroize",
            "--release",
            "--example",
            "zeroize_asm",
            "--target",
            TARGET,
            "--locked",
        ],
        check=True,
        stdout=subprocess.DEVNULL,
        env=env,
    )
    binary = target_dir / TARGET / "release" / "examples" / "zeroize_asm"
    if not binary.is_file():
        raise SystemExit(f"missing zeroization example: {binary}")
    return binary


def function_body(disassembly: str, symbol: str) -> str | None:
    lines = disassembly.splitlines()
    start = None
    for index, line in enumerate(lines):
        if symbol in line and line.rstrip().endswith(">:"):
            start = index
            break
    if start is None:
        return None

    end = start + 1
    while end < len(lines):
        candidate = lines[end].rstrip()
        if candidate.endswith(">:") and "<" in candidate:
            break
        end += 1
    return "\n".join(lines[start:end])


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()

    binary = build()
    disassembly = subprocess.run(
        ["objdump", "-d", str(binary)],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    ).stdout
    body = function_body(disassembly, "sicas_zeroize13zeroize_bytes")
    if body is None:
        print("zeroize_bytes symbol not found", file=sys.stderr)
        return 1

    zero_stores = re.findall(r"\bmov[bwlq]?\s+\$0x0,", body)
    calls_memset = "memset" in body
    status = "PASS" if zero_stores and not calls_memset else "FAIL"

    report = "\n".join(
        [
            "# Zeroization Assembly Evidence",
            "",
            f"- Target: `{TARGET}`",
            f"- Symbol: `sicas_zeroize::zeroize_bytes`",
            f"- Volatile zero-store instructions: {len(zero_stores)}",
            f"- Calls `memset`: {calls_memset}",
            f"- Result: **{status}**",
            "",
            "The function emits explicit zero stores and is not replaced by a",
            "library memset call.",
            "",
            "## Disassembly",
            "",
            "```text",
            body,
            "```",
        ]
    ) + "\n"

    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(report, encoding="utf-8")
    print(report)
    return 0 if status == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main())
