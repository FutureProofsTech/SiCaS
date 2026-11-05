#!/usr/bin/env python3
"""Tests for the reference key-plane constant-time gate."""

from __future__ import annotations

import pathlib
import tempfile
import unittest

import reference_ct_check as checker

GOOD_SOURCE = """
#[inline(never)]
fn spread_bit(value: u64, bit_mask: u64) -> u64 {
    u64::from(value & bit_mask != 0).wrapping_neg()
}

fn masked_word_planes(word: &Masked<u32>, entropy: &mut impl EntropySource) -> Result<Planes, EntropyError> {
    for (plane, bit_mask) in planes.iter_mut().zip(BIT_MASKS) {
        let share0_bits = spread_bit(u64::from(*share0), bit_mask);
        let share1_bits = spread_bit(u64::from(*share1), bit_mask);
        let mask = entropy.mask_u64()?;
        *plane = Plane::from_parts(share0_bits ^ mask, share1_bits ^ mask);
    }
    Ok(planes)
}
"""

BAD_SOURCE = """
fn spread_bit(value: u64, bit_mask: u64) -> u64 {
    u64::from(value & bit_mask != 0).wrapping_neg()
}

fn masked_word_planes(word: &Masked<u32>, entropy: &mut impl EntropySource) -> Result<Planes, EntropyError> {
    for bit in 0..32 {
        let shift = u32::try_from(bit).unwrap_or(0);
        let share0_bits = if (*share0 >> shift) & 1 == 1 { u64::MAX } else { 0 };
        let mask = entropy.mask_u64()?;
        planes[bit] = Plane::from_parts(share0_bits ^ mask, 0);
    }
    Ok(planes)
}
"""

GOOD_ASSEMBLY = """
.section .text._RNvNtCs_15sicas_reference8chacha2010spread_bit,"ax",@progbits
.globl _RNvNtCs_15sicas_reference8chacha2010spread_bit
.type _RNvNtCs_15sicas_reference8chacha2010spread_bit,@function
_RNvNtCs_15sicas_reference8chacha2010spread_bit:
    setne %al
    negq %rax
    retq
.size _RNvNtCs_15sicas_reference8chacha2010spread_bit, .-_RNvNtCs_15sicas_reference8chacha2010spread_bit
"""

BAD_ASSEMBLY = """
.section .text._RNvNtCs_15sicas_reference8chacha2010spread_bit,"ax",@progbits
.globl _RNvNtCs_15sicas_reference8chacha2010spread_bit
.type _RNvNtCs_15sicas_reference8chacha2010spread_bit,@function
_RNvNtCs_15sicas_reference8chacha2010spread_bit:
    je .Lbranch
    jmp .Ldone
.Lbranch:
    retq
.size _RNvNtCs_15sicas_reference8chacha2010spread_bit, .-_RNvNtCs_15sicas_reference8chacha2010spread_bit
"""


class ReferenceCtTests(unittest.TestCase):
    def test_good_source_passes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "chacha20.rs"
            path.write_text(GOOD_SOURCE, encoding="utf-8")
            self.assertEqual(checker.check_source(path), [])

    def test_share_branch_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "chacha20.rs"
            path.write_text(BAD_SOURCE, encoding="utf-8")
            errors = checker.check_source(path)
            self.assertTrue(any("forbidden conditional" in error for error in errors))
            self.assertTrue(any("forbidden variable shift" in error for error in errors))

    def test_branchless_assembly_passes(self) -> None:
        errors, branches = checker.check_assembly(
            GOOD_ASSEMBLY, "x86_64-unknown-linux-gnu"
        )
        self.assertEqual(errors, [])
        self.assertEqual(branches, [])

    def test_branching_assembly_is_rejected(self) -> None:
        errors, branches = checker.check_assembly(
            BAD_ASSEMBLY, "x86_64-unknown-linux-gnu"
        )
        self.assertTrue(errors)
        self.assertIn("je", branches)

    def test_missing_symbol_is_rejected(self) -> None:
        errors, _ = checker.check_assembly("", "x86_64-unknown-linux-gnu")
        self.assertTrue(errors)

    def test_target_branch_patterns(self) -> None:
        self.assertEqual(
            checker.find_branches("cbz x0, .L1", "aarch64-unknown-linux-gnu"),
            ["cbz"],
        )
        self.assertEqual(
            checker.find_branches("beq a0, a1, .L1", "riscv32imac-unknown-none-elf"),
            ["beq"],
        )


if __name__ == "__main__":
    unittest.main()
