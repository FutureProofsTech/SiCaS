#!/usr/bin/env python3
"""Constant-time gate for the bitsliced ChaCha20 key-plane primitive.

The gate has two layers:

1. Source invariants for `masked_word_planes` and `spread_bit`: no conditional
   statements, no variable shifts on shares, and the branchless
   `u64::from(bool).wrapping_neg()` spread must be present.
2. An assembly matrix: `spread_bit` is `#[inline(never)]`, so it is emitted at
   every optimization level and on every supported target. The gate rejects any
   conditional branch in the emitted function.

This is a targeted primitive gate, not a proof of the entire reference
composition. It exists because RT-01 was a branch on a key-share bit that the
core-only assembly matrix did not inspect.
"""

from __future__ import annotations

import argparse
import os
import pathlib
import re
import subprocess
import sys

TARGETS = [
    (
        "x86_64-unknown-linux-gnu",
        re.compile(r"\b(j[a-z]+|loop[a-z]*)\b"),
    ),
    (
        "aarch64-unknown-linux-gnu",
        re.compile(r"\b(b\.[a-z]+|cbz|cbnz|tbz|tbnz)\b"),
    ),
    (
        "thumbv7em-none-eabihf",
        re.compile(
            r"\b(b(?:eq|ne|cs|hs|cc|lo|mi|pl|vs|vc|hi|ls|ge|lt|gt|le)|cbz|cbnz|tbz|tbnz)\b"
        ),
    ),
    (
        "riscv32imac-unknown-none-elf",
        re.compile(
            r"\b(?:c\.)?(?:beq|bne|blt|bge|bltu|bgeu|beqz|bnez|blez|bgez|bltz|bgtz|bgt|ble|bgtu|bleu)\b"
        ),
    ),
]

OPT_LEVELS = ["0", "1", "2", "3", "s"]
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
    source = path.read_text(encoding="utf-8")
    masked = function_body(source, "masked_word_planes")
    spread = function_body(source, "spread_bit")
    if masked is None:
        errors.append(f"{path}: missing masked_word_planes")
    if spread is None:
        errors.append(f"{path}: missing spread_bit")
        return errors

    for name, body in (("masked_word_planes", masked), ("spread_bit", spread)):
        for token in FORBIDDEN_TOKENS:
            if token in body:
                errors.append(f"{path}: forbidden conditional token {token!r} in {name}")
        for token in FORBIDDEN_SHIFTS:
            if token in body:
                errors.append(f"{path}: forbidden variable shift {token!r} in {name}")

    if spread.count("wrapping_neg") < 1:
        errors.append(f"{path}: spread_bit must use wrapping_neg")
    if spread.count("!= 0") < 1:
        errors.append(f"{path}: spread_bit must use a branchless bit test")
    if masked.count("spread_bit") < 2:
        errors.append(f"{path}: masked_word_planes must use spread_bit for both shares")
    return errors


def extract_function(assembly: str, symbol: str) -> str | None:
    lines = assembly.splitlines()
    start = None
    for index, line in enumerate(lines):
        if ".section" in line and symbol in line:
            start = index
            break
    if start is None:
        for index, line in enumerate(lines):
            if ".type" in line and symbol in line:
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


def find_branches(section: str, target: str) -> list[str]:
    pattern = dict(TARGETS)[target]
    return sorted({match.group(0) for match in pattern.finditer(section)})


def build_library(
    root: pathlib.Path,
    target: str,
    opt_level: str,
    target_dir: pathlib.Path,
) -> pathlib.Path:
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(target_dir)
    subprocess.run(
        [
            "cargo",
            "rustc",
            "-p",
            "sicas-reference",
            "--lib",
            "--target",
            target,
            "--locked",
            "--",
            "--emit=asm",
            "-C",
            f"opt-level={opt_level}",
        ],
        cwd=root,
        env=env,
        check=True,
        stdout=subprocess.DEVNULL,
    )
    files = sorted(
        target_dir.rglob("sicas_reference-*.s"),
        key=lambda path: path.stat().st_mtime_ns,
    )
    if not files:
        raise SystemExit(f"no library assembly for {target} at opt-level={opt_level}")
    return files[-1]


def check_assembly(assembly: str, target: str) -> tuple[list[str], list[str]]:
    body = extract_function(assembly, "spread_bit")
    if body is None:
        return ["spread_bit symbol not found in assembly"], []
    branches = find_branches(body, target)
    if branches:
        return [f"conditional branches in spread_bit: {', '.join(branches)}"], branches
    return [], []


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
        help="inspect x86_64 opt-level=0 assembly in addition to the source checks",
    )
    parser.add_argument(
        "--matrix",
        action="store_true",
        help="inspect all targets and optimization levels (implies --assembly)",
    )
    parser.add_argument(
        "--target-dir",
        type=pathlib.Path,
        default=pathlib.Path("target/reference-ct"),
    )
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()

    root = pathlib.Path.cwd()
    errors = check_source(args.path)
    rows: list[tuple[str, str, str, str]] = []

    if args.matrix:
        configs = [(target, opt) for target, _ in TARGETS for opt in OPT_LEVELS]
    elif args.assembly:
        configs = [("x86_64-unknown-linux-gnu", "0")]
    else:
        configs = []

    for target, opt_level in configs:
        target_dir = args.target_dir / target / f"opt-{opt_level}"
        try:
            assembly_path = build_library(root, target, opt_level, target_dir)
        except (OSError, subprocess.CalledProcessError) as error:
            errors.append(f"{target} opt-{opt_level}: assembly build failed: {error}")
            rows.append((target, opt_level, "BUILD-FAIL", ""))
            continue
        assembly_errors, branches = check_assembly(
            assembly_path.read_text(encoding="utf-8"), target
        )
        errors.extend(f"{target} opt-{opt_level}: {error}" for error in assembly_errors)
        rows.append(
            (
                target,
                opt_level,
                "FAIL" if assembly_errors else "pass",
                ", ".join(branches) if branches else "none",
            )
        )

    if args.output:
        lines = [
            "# Reference Key-Plane Assembly Evidence",
            "",
            "Generated by `tools/reference_ct_check.py`.",
            "",
            "| Target | opt-level | Conditional branches in `spread_bit` | Result |",
            "|---|---|---|---|",
        ]
        lines.extend(f"| {target} | {opt} | {branches} | {status} |" for target, opt, status, branches in rows)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text("\n".join(lines) + "\n", encoding="utf-8")

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print(f"reference key-plane gate passed: {args.path} ({len(rows)} assembly configurations)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
