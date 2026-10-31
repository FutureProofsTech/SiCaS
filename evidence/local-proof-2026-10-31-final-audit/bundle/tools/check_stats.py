#!/usr/bin/env python3
"""Differentially validate Rust statistics against an independent Python model."""

from __future__ import annotations

import argparse
import datetime as dt
import math
import pathlib
import subprocess
import sys

CASES = 200


def mean(values: list[float]) -> float:
    return sum(values) / len(values)


def variance(values: list[float]) -> float:
    average = mean(values)
    return sum((value - average) ** 2 for value in values) / (len(values) - 1)


def welch_t(a: list[float], b: list[float]) -> float:
    mean_a = mean(a)
    mean_b = mean(b)
    variance_a = variance(a)
    variance_b = variance(b)
    denominator = math.sqrt(variance_a / len(a) + variance_b / len(b))
    if denominator <= 0.0:
        return 0.0
    return (mean_a - mean_b) / denominator


def pearson(x: list[float], y: list[float]) -> float:
    if len(x) != len(y) or len(x) < 2:
        return math.nan
    count = len(x)
    sum_x = sum(x)
    sum_y = sum(y)
    sum_x2 = sum(value * value for value in x)
    sum_y2 = sum(value * value for value in y)
    cross = sum(a * b for a, b in zip(x, y))
    numerator = count * cross - sum_x * sum_y
    denominator_x = max(count * sum_x2 - sum_x * sum_x, 0.0)
    denominator_y = max(count * sum_y2 - sum_y * sum_y, 0.0)
    denominator = math.sqrt(denominator_x * denominator_y)
    if denominator <= 0.0:
        return math.nan
    return numerator / denominator


def parse_values(text: str) -> list[float]:
    return [float(value) for value in text.split(",")]


def close(actual: float, expected: float) -> bool:
    if math.isnan(actual) or math.isnan(expected):
        return math.isnan(actual) and math.isnan(expected)
    return math.isclose(actual, expected, rel_tol=1e-12, abs_tol=1e-12)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()

    output = subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "--release",
            "--locked",
            "-p",
            "sicas-harness",
            "--example",
            "stats_fixture",
        ],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    ).stdout

    lines = [line for line in output.splitlines() if line.strip()]
    if len(lines) != CASES:
        print(f"expected {CASES} cases, got {len(lines)}", file=sys.stderr)
        return 1

    max_error = 0.0
    failures: list[str] = []
    for line in lines:
        fields = line.split("|")
        if len(fields) != 9:
            failures.append(f"malformed line: {line}")
            continue
        case, text_a, text_b, mean_a, mean_b, variance_a, variance_b, t_value, r_value = fields
        a = parse_values(text_a)
        b = parse_values(text_b)

        expected = [
            mean(a),
            mean(b),
            variance(a),
            variance(b),
            welch_t(a, b),
            pearson(a, b),
        ]
        actual = [
            float(mean_a),
            float(mean_b),
            float(variance_a),
            float(variance_b),
            float(t_value),
            float(r_value),
        ]
        for name, observed, reference in zip(
            ["mean_a", "mean_b", "variance_a", "variance_b", "welch_t", "pearson"],
            actual,
            expected,
        ):
            if not close(observed, reference):
                failures.append(
                    f"case {case} {name}: rust={observed!r} python={reference!r}"
                )
            elif not math.isnan(observed):
                max_error = max(max_error, abs(observed - reference))

    status = "PASS" if not failures else "FAIL"
    lines_out = [
        "# Statistics Differential Evidence",
        "",
        f"- Date: {dt.date.today().isoformat()}",
        f"- Cases: {len(lines)}",
        f"- Max absolute error: {max_error:.3e}",
        f"- Result: {status}",
        "",
        "Rust `sicas-harness::stats` results were compared with an independent Python",
        "implementation of the same formulas over deterministic random inputs.",
    ]
    report = "\n".join(lines_out) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(report, encoding="utf-8")
    print(report)

    if failures:
        for failure in failures:
            print(failure, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
