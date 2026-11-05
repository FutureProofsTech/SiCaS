#!/usr/bin/env python3
"""Tests for the reference key-plane constant-time gate."""

from __future__ import annotations

import pathlib
import tempfile
import unittest

import reference_ct_check as checker

GOOD_SOURCE = """
fn masked_word_planes(word: &Masked<u32>, entropy: &mut impl EntropySource) -> Result<Planes, EntropyError> {
    for (plane, bit_mask) in planes.iter_mut().zip(BIT_MASKS) {
        let share0_bits = u64::from(u64::from(*share0) & bit_mask != 0).wrapping_neg();
        let share1_bits = u64::from(u64::from(*share1) & bit_mask != 0).wrapping_neg();
        let mask = entropy.mask_u64()?;
        *plane = Plane::from_parts(share0_bits ^ mask, share1_bits ^ mask);
    }
    Ok(planes)
}
"""

BAD_SOURCE = """
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
.type _RNvNtCs_15sicas_reference8chacha2018masked_word_planes,@function
_RNvNtCs_15sicas_reference8chacha2018masked_word_planes:
    setne %al
    setne %bl
    retq
.size _RNvNtCs_15sicas_reference8chacha2018masked_word_planes, .-_RNvNtCs_15sicas_reference8chacha2018masked_word_planes
"""

BAD_ASSEMBLY = """
.type _RNvNtCs_15sicas_reference8chacha2018masked_word_planes,@function
_RNvNtCs_15sicas_reference8chacha2018masked_word_planes:
    je .Lbranch
    jmp .Ldone
.Lbranch:
    retq
.size _RNvNtCs_15sicas_reference8chacha2018masked_word_planes, .-_RNvNtCs_15sicas_reference8chacha2018masked_word_planes
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

    def test_setne_assembly_passes(self) -> None:
        self.assertEqual(checker.check_assembly_text(GOOD_ASSEMBLY), [])

    def test_branching_assembly_is_rejected(self) -> None:
        errors = checker.check_assembly_text(BAD_ASSEMBLY)
        self.assertTrue(any("setne" in error for error in errors))

    def test_missing_symbol_is_rejected(self) -> None:
        self.assertTrue(checker.check_assembly_text(""))


if __name__ == "__main__":
    unittest.main()
