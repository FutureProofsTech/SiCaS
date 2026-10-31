#!/usr/bin/env python3
"""Inspect generated assembly for conditional branches in constant-time gadgets.

This is supporting evidence, not a formal proof. The default mode builds
`sicas-core` for each target at `-C opt-level=0` and checks the library symbols.
`--matrix` additionally builds the `ct_asm` example across optimization levels,
codegen-unit counts, and panic strategies on std-capable targets. `--lto` builds
linked x86_64 binaries with thin/fat LTO and inspects the disassembly.
"""

from __future__ import annotations

import argparse
import os
import pathlib
import re
import subprocess
import sys

TARGETS = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "thumbv7em-none-eabihf",
    "riscv32imac-unknown-none-elf",
]

STD_TARGETS = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
]

LIB_FUNCTIONS = ["eq_u32", "select_u32", "is_zero_u32"]
EXAMPLE_FUNCTIONS = ["asm_eq_u32", "asm_select_u32", "asm_is_zero_u32"]

MATRIX_CONFIGS = [
    ("debug-o0", False, "0", None, None),
    ("debug-o1", False, "1", None, None),
    ("release-o2", True, "2", None, None),
    ("release-o3", True, "3", None, None),
    ("release-os", True, "s", None, None),
    ("release-o3-cgu1", True, "3", "1", None),
    ("release-o3-panic-abort", True, "3", None, "abort"),
    ("release-oz", True, "z", None, None),
]

LTO_CONFIGS = [
    ("release-o3-lto-thin", "thin", "3"),
    ("release-o3-lto-fat", "fat", "3"),
    ("release-oz-lto-fat", "fat", "z"),
]

BRANCH_PATTERNS = {
    "x86_64-unknown-linux-gnu": re.compile(r"\b(j[a-z]+|loop[a-z]*)\b"),
    "aarch64-unknown-linux-gnu": re.compile(r"\b(b\.[a-z]+|cbz|cbnz|tbz|tbnz)\b"),
    "thumbv7em-none-eabihf": re.compile(
        r"\b(b(?:eq|ne|cs|hs|cc|lo|mi|pl|vs|vc|hi|ls|ge|lt|gt|le)|cbz|cbnz|tbz|tbnz)\b"
    ),
    "riscv32imac-unknown-none-elf": re.compile(
        r"\b(?:c\.)?(?:beq|bne|blt|bge|bltu|bgeu|beqz|bnez|blez|bgez|bltz|bgtz|bgt|ble|bgtu|bleu)\b"
    ),
}


def run(command: list[str], *, env: dict[str, str] | None = None) -> None:
    subprocess.run(
        command,
        check=True,
        stdout=subprocess.DEVNULL,
        env=env,
    )


def build_library(target: str) -> pathlib.Path:
    run(
        [
            "cargo",
            "rustc",
            "-p",
            "sicas-core",
            "--lib",
            "--target",
            target,
            "--locked",
            "--",
            "--emit=asm",
            "-C",
            "opt-level=0",
        ]
    )
    deps = pathlib.Path("target") / target / "debug" / "deps"
    files = sorted(deps.glob("sicas_core-*.s"))
    if not files:
        raise SystemExit(f"no assembly file for {target} in {deps}")
    return files[-1]


def build_example(
    name: str,
    release: bool,
    opt_level: str,
    codegen_units: str | None,
    panic: str | None,
    target: str,
) -> pathlib.Path:
    target_dir = pathlib.Path("target") / "asm-matrix" / name / target
    command = [
        "cargo",
        "rustc",
        "-p",
        "sicas-core",
        "--example",
        "ct_asm",
        "--target",
        target,
        "--locked",
    ]
    if release:
        command.append("--release")
    command.extend(
        [
            "--",
            "--emit=asm",
            "-C",
            f"opt-level={opt_level}",
            "-C",
            "linker=/bin/true",
        ]
    )
    if codegen_units is not None:
        command.extend(["-C", f"codegen-units={codegen_units}"])
    if panic is not None:
        command.extend(["-C", f"panic={panic}"])

    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(target_dir)
    run(command, env=env)

    files = sorted(target_dir.rglob("ct_asm*.s"))
    if not files:
        raise SystemExit(f"no example assembly for {name} on {target}")
    return files[-1]


def build_lto_binary(name: str, lto: str, opt_level: str, target: str) -> pathlib.Path:
    target_dir = pathlib.Path("target") / "asm-matrix" / name / target
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(target_dir)
    env["CARGO_PROFILE_RELEASE_STRIP"] = "none"
    env["RUSTFLAGS"] = (
        f"-C lto={lto} -C embed-bitcode=yes -C opt-level={opt_level}"
    )
    run(
        [
            "cargo",
            "build",
            "-p",
            "sicas-core",
            "--example",
            "ct_asm",
            "--release",
            "--target",
            target,
            "--locked",
        ],
        env=env,
    )

    binary = target_dir / target / "release" / "examples" / "ct_asm"
    if not binary.is_file():
        raise SystemExit(f"no LTO binary for {name} on {target}")
    return binary


def extract_function(assembly: str, function: str, marker: str) -> str | None:
    lines = assembly.splitlines()
    start = None
    for index, line in enumerate(lines):
        if ".section" in line and marker in line and function in line:
            start = index
            break
    if start is None:
        return None

    end = start + 1
    while end < len(lines) and ".size" not in lines[end]:
        end += 1
    return "\n".join(lines[start : end + 1])


def extract_disassembly_function(disassembly: str, function: str) -> str | None:
    lines = disassembly.splitlines()
    start = None
    for index, line in enumerate(lines):
        if line.rstrip().endswith(":") and function in line and "<" in line:
            start = index
            break
    if start is None:
        return None

    end = start + 1
    while end < len(lines):
        candidate = lines[end].rstrip()
        if candidate.endswith(":") and "<" in candidate and ">:" in candidate:
            break
        end += 1
    return "\n".join(lines[start:end])


def find_branches(section: str, target: str) -> list[str]:
    pattern = BRANCH_PATTERNS[target]
    branches = []
    for match in pattern.finditer(section):
        instruction = match.group(0)
        if target == "x86_64-unknown-linux-gnu" and instruction == "jmp":
            continue
        branches.append(instruction)
    return sorted(set(branches))


def add_row(
    rows: list[tuple[str, str, str, str, str]],
    failures: list[str],
    config: str,
    target: str,
    function: str,
    section: str | None,
) -> None:
    if section is None:
        rows.append((config, target, function, "MISSING", "FAIL"))
        failures.append(f"{config}/{target}/{function}: missing")
        return
    branches = find_branches(section, target)
    status = "pass" if not branches else "FAIL"
    if branches:
        failures.append(f"{config}/{target}/{function}: {', '.join(branches)}")
    rendered = ", ".join(branches) if branches else "none"
    rows.append((config, target, function, rendered, status))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=pathlib.Path)
    parser.add_argument(
        "--matrix",
        action="store_true",
        help="also check optimized example builds on std-capable targets",
    )
    parser.add_argument(
        "--lto",
        action="store_true",
        help="also check linked x86_64 LTO binaries",
    )
    args = parser.parse_args()

    rows: list[tuple[str, str, str, str, str]] = []
    failures: list[str] = []

    for target in TARGETS:
        assembly = build_library(target).read_text(encoding="utf-8")
        for function in LIB_FUNCTIONS:
            section = extract_function(assembly, function, "sicas_core2ct")
            add_row(rows, failures, "debug-o0", target, function, section)

    if args.matrix:
        for config in MATRIX_CONFIGS:
            name, release, opt_level, codegen_units, panic = config
            for target in STD_TARGETS:
                assembly = build_example(
                    name, release, opt_level, codegen_units, panic, target
                ).read_text(encoding="utf-8")
                for function in EXAMPLE_FUNCTIONS:
                    section = extract_function(assembly, function, "ct_asm")
                    add_row(rows, failures, name, target, function, section)

    if args.lto:
        for name, lto, opt_level in LTO_CONFIGS:
            target = "x86_64-unknown-linux-gnu"
            binary = build_lto_binary(name, lto, opt_level, target)
            disassembly = subprocess.run(
                ["objdump", "-d", str(binary)],
                check=True,
                text=True,
                stdout=subprocess.PIPE,
            ).stdout
            for function in EXAMPLE_FUNCTIONS:
                section = extract_disassembly_function(disassembly, function)
                add_row(rows, failures, name, target, function, section)

    lines = [
        "# Constant-Time Assembly Evidence",
        "",
        "Generated by `tools/ct_asm_check.py`.",
        "",
        "Default rows build the library at `-C opt-level=0` for four targets.",
        "Matrix rows build the `ct_asm` example on std-capable targets.",
        "LTO rows inspect linked x86_64 binaries.",
        "",
        "| Config | Target | Function | Conditional branches | Result |",
        "|---|---|---|---|---|",
    ]
    for config, target, function, branches, status in rows:
        lines.append(
            f"| {config} | {target} | `{function}` | {branches} | {status} |"
        )
    report = "\n".join(lines) + "\n"

    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(report, encoding="utf-8")
    print(report)
    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
