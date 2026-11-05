#!/usr/bin/env python3
"""Tests for the workflow action-pin checker."""

from __future__ import annotations

import pathlib
import tempfile
import unittest

import check_workflow_pins as checker

SHA = "0123456789abcdef0123456789abcdef01234567"


class WorkflowPinTests(unittest.TestCase):
    def test_pinned_action_passes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "ci.yml"
            path.write_text(
                f"jobs:\n  test:\n    steps:\n      - uses: actions/checkout@{SHA}\n",
                encoding="utf-8",
            )
            self.assertEqual(checker.check_file(path), [])

    def test_movable_tag_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "ci.yml"
            path.write_text(
                "jobs:\n  test:\n    steps:\n      - uses: actions/checkout@v4\n",
                encoding="utf-8",
            )
            errors = checker.check_file(path)
            self.assertEqual(len(errors), 1)
            self.assertIn("not pinned", errors[0])

    def test_local_and_docker_references_are_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "ci.yml"
            path.write_text(
                "jobs:\n  test:\n    steps:\n"
                "      - uses: ./.github/actions/local\n"
                "      - uses: docker://alpine:3\n",
                encoding="utf-8",
            )
            self.assertEqual(checker.check_file(path), [])


if __name__ == "__main__":
    unittest.main()
