#!/usr/bin/env python3
"""Targeted constant-time gate for bitsliced ChaCha20 key-plane extraction.

The gate has two layers:

1. A source invariant: `masked_word_planes` must not contain a conditional
   statement or a variable shift on a share, and must use the branchless
   `u64::from(bool).wrapping_neg()` spread.
2. An optional `opt-level=0` assembly check: the compiled function must contain
   at least two `setne`-style bit selections, which is how the branchless spread
   is lowered on x86_64.

This is a regression gate for the RT-01 defect, not a proof for every compiler
or target.
"""

from __future__ import annotations

import argparse
import os
import pathlib
import subprocess
import sys

FORBIDDEN_TOKENS = ("if ", "match ", "while ", "loop ")
FORBIDDEN_SHIFTS = (">>", "<<")


def function_body(source: str, name: str) -> str | None:
    marker = f"fn {name}("
    start = source.find(marker)
    if start < 0:
        return None
    brace = source.find("{", start)
    if brace < 0:
        return None
    depth = 0
    for index in range(brace, len(source)):
        character = source[index]
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
            if depth == 0:
                return source[start : index + 1]
    return None


def check_source(path: pathlib.Path) -> list[str]:
    errors: list[str] = []
    body = function_body(path.read_text(encoding="utf-8"), "masked_word_planes")
    if body is None:
        return [f"{path}: missing masked_word_planes"]
    for token in FORBIDDEN_TOKENS:
        if token in body:
            errors.append(f"{path}: forbidden conditional token {token!r} in masked_word_planes")
    for token in FORBIDDEN_SHIFTS:
        if token in body:
            errors.append(f"{path}: forbidden variable shift {token!r} in masked_word_planes")
    if body.count("wrapping_neg") < 2:
        errors.append(f"{path}: expected two wrapping_neg bit-spread operations")
    if body.count("!= 0") < 2:
        errors.append(f"{path}: expected two branchless bit tests")
    return errors


def extract_function(assembly: str, symbol: str) -> str | None:
    lines = assembly.splitlines()
    start = None
    for index, line in enumerate(lines):
        if symbol in line and line.rstrip().endswith("@function"):
            start = index
            break
    if start is None:
        return None
    end = start + 1
    while end < len(lines):
        if ".size" in lines[end] and symbol in lines[end]:
            break
        end += 1
    return "\n".join(lines[start : end + 1])


def check_assembly_text(assembly: str) -> list[str]:
    body = extract_function(assembly, "masked_word_planes")
    if body is None:
        return ["masked_word_planes symbol not found in assembly"]
    if body.count("setne") < 2:
        return [
            "masked_word_planes has fewer than two setne bit selections; "
            "the branchless spread may have regressed"
        ]
    return []


def build_assembly(root: pathlib.Path, target_dir: pathlib.Path) -> pathlib.Path:
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(target_dir)
    subprocess.run(
        [
            "cargo",
            "rustc",
            "-p",
            "sicas-reference",
            "--example",
            "benchmark",
            "--locked",
            "--",
            "--emit=asm",
            "-C",
            "opt-level=0",
        ],
        cwd=root,
        env=env,
        check=True,
        stdout=subprocess.DEVNULL,
    )
    examples = target_dir / "debug" / "examples"
    files = sorted(examples.glob("benchmark-*.s"), key=lambda path: path.stat().st_mtime_ns)
    if not files:
        raise SystemExit(f"no benchmark assembly in {examples}")
    return files[-1]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "path",
        type=pathlib.Path,
        nargs="?",
        default=pathlib.Path("crates/sicas-reference/src/chacha20.rs"),
    )
    parser.add_argument(
        "--assembly",
        action="store_true",
        help="also inspect the opt-level=0 assembly for setne-style bit selection",
    )
    parser.add_argument(
        "--target-dir",
        type=pathlib.Path,
        default=pathlib.Path("target/reference-ct"),
    )
    args = parser.parse_args()

    root = pathlib.Path.cwd()
    errors = check_source(args.path)
    if args.assembly:
        try:
            assembly_path = build_assembly(root, args.target_dir)
        except (OSError, subprocess.CalledProcessError) as error:
            errors.append(f"assembly build failed: {error}")
        else:
            errors.extend(check_assembly_text(assembly_path.read_text(encoding="utf-8")))

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print(f"reference key-plane extraction gate passed: {args.path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
