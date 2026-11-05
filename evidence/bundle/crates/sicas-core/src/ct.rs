//! Constant-time comparison, selection, swapping, and table lookup.

use crate::Choice;

/// Returns a `Choice` that is true when `x` is zero.
pub const fn is_zero_u8(x: u8) -> Choice {
    let folded = x | x.wrapping_neg();
    Choice::from_raw_u8((folded >> 7) ^ 1)
}

/// Returns a `Choice` that is true when `x` is zero.
pub const fn is_zero_u16(x: u16) -> Choice {
    let folded = x | x.wrapping_neg();
    Choice::from_raw_u16((folded >> 15) ^ 1)
}

/// Returns a `Choice` that is true when `x` is zero.
pub const fn is_zero_u32(x: u32) -> Choice {
    let folded = x | x.wrapping_neg();
    Choice::from_raw_u32((folded >> 31) ^ 1)
}

/// Returns a `Choice` that is true when `x` is zero.
pub const fn is_zero_u64(x: u64) -> Choice {
    let folded = x | x.wrapping_neg();
    Choice::from_raw_u64((folded >> 63) ^ 1)
}

/// Returns a `Choice` that is true when `x` is zero.
pub const fn is_zero_u128(x: u128) -> Choice {
    let folded = x | x.wrapping_neg();
    Choice::from_raw_u128((folded >> 127) ^ 1)
}

/// Compares two bytes in constant time.
pub const fn eq_u8(a: u8, b: u8) -> Choice {
    is_zero_u8(a ^ b)
}

/// Compares two `u16` values in constant time.
pub const fn eq_u16(a: u16, b: u16) -> Choice {
    is_zero_u16(a ^ b)
}

/// Compares two `u32` values in constant time.
pub const fn eq_u32(a: u32, b: u32) -> Choice {
    is_zero_u32(a ^ b)
}

/// Compares two `u64` values in constant time.
pub const fn eq_u64(a: u64, b: u64) -> Choice {
    is_zero_u64(a ^ b)
}

/// Compares two `u128` values in constant time.
pub const fn eq_u128(a: u128, b: u128) -> Choice {
    is_zero_u128(a ^ b)
}

/// Selects `a` when `choice` is true, otherwise `b`, in constant time.
#[must_use]
pub const fn select_u8(choice: Choice, a: u8, b: u8) -> u8 {
    b ^ (choice.mask_u8() & (a ^ b))
}

/// Selects `a` when `choice` is true, otherwise `b`, in constant time.
#[must_use]
pub const fn select_u16(choice: Choice, a: u16, b: u16) -> u16 {
    b ^ (choice.mask_u16() & (a ^ b))
}

/// Selects `a` when `choice` is true, otherwise `b`, in constant time.
#[must_use]
pub const fn select_u32(choice: Choice, a: u32, b: u32) -> u32 {
    b ^ (choice.mask_u32() & (a ^ b))
}

/// Selects `a` when `choice` is true, otherwise `b`, in constant time.
#[must_use]
pub const fn select_u64(choice: Choice, a: u64, b: u64) -> u64 {
    b ^ (choice.mask_u64() & (a ^ b))
}

/// Selects `a` when `choice` is true, otherwise `b`, in constant time.
#[must_use]
pub const fn select_u128(choice: Choice, a: u128, b: u128) -> u128 {
    b ^ (choice.mask_u128() & (a ^ b))
}

/// Swaps two `u32` values in constant time when `choice` is true.
pub const fn swap_u32(a: &mut u32, b: &mut u32, choice: Choice) {
    let difference = choice.mask_u32() & (*a ^ *b);
    *a ^= difference;
    *b ^= difference;
}

/// Looks up `index` in a 256-byte table without a secret-dependent memory access.
///
/// The complete table is scanned. The returned value is selected in constant
/// time, so the memory-access pattern does not depend on `index`.
#[must_use]
pub fn lookup_u8(table: &[u8; 256], index: u8) -> u8 {
    let mut result = 0_u8;
    let mut candidate = 0_u8;

    loop {
        let selected = eq_u8(candidate, index);
        let value = table.get(usize::from(candidate)).copied().unwrap_or(0_u8);
        result |= value & selected.mask_u8();

        if candidate == u8::MAX {
            break;
        }

        candidate = candidate.wrapping_add(1);
    }

    result
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::integer_division,
        clippy::as_conversions
    )]

    use super::*;

    #[test]
    fn zero_detection_is_exact_for_all_bytes() {
        // Native runs are exhaustive; Miri samples the same code paths because
        // interpreting 65k+ cases is prohibitively slow.
        let step = if cfg!(miri) { 64 } else { 1 };
        for value in (u8::MIN..=u8::MAX).step_by(step) {
            let expected = u8::from(value == 0);
            assert_eq!(is_zero_u8(value).as_u8(), expected);
        }
    }

    #[test]
    fn equality_is_exact_for_all_byte_pairs() {
        let step = if cfg!(miri) { 64 } else { 1 };
        for a in (u8::MIN..=u8::MAX).step_by(step) {
            for b in (u8::MIN..=u8::MAX).step_by(step) {
                let expected = u8::from(a == b);
                assert_eq!(eq_u8(a, b).as_u8(), expected);
            }
        }
    }

    #[test]
    fn selection_is_exact_for_all_byte_pairs() {
        let step = if cfg!(miri) { 64 } else { 1 };
        for a in (u8::MIN..=u8::MAX).step_by(step) {
            for b in (u8::MIN..=u8::MAX).step_by(step) {
                assert_eq!(select_u8(Choice::from_bool(true), a, b), a);
                assert_eq!(select_u8(Choice::from_bool(false), a, b), b);
            }
        }
    }

    #[test]
    fn swap_swaps_only_when_selected() {
        let mut a = 0x1234_5678_u32;
        let mut b = 0x9abc_def0_u32;
        let original_a = a;
        let original_b = b;

        swap_u32(&mut a, &mut b, Choice::from_bool(false));
        assert_eq!((a, b), (original_a, original_b));

        swap_u32(&mut a, &mut b, Choice::from_bool(true));
        assert_eq!((a, b), (original_b, original_a));
    }

    #[test]
    fn lookup_scans_the_whole_table_correctly() {
        let table: [u8; 256] = core::array::from_fn(|index| u8::try_from(index).unwrap());
        let step = if cfg!(miri) { 64 } else { 1 };

        for index in (u8::MIN..=u8::MAX).step_by(step) {
            assert_eq!(lookup_u8(&table, index), table[usize::from(index)]);
        }
    }

    #[test]
    fn wide_equality_and_selection_work() {
        assert_eq!(eq_u32(0, 0).as_u8(), 1);
        assert_eq!(eq_u32(0, 1).as_u8(), 0);
        assert_eq!(is_zero_u32(0x8000_0000).as_u8(), 0);
        assert_eq!(is_zero_u32(u32::MAX).as_u8(), 0);
        assert_eq!(eq_u64(u64::MAX, u64::MAX).as_u8(), 1);
        assert_eq!(eq_u64(u64::MAX, 0).as_u8(), 0);
        assert_eq!(is_zero_u64(0x8000_0000_0000_0000).as_u8(), 0);
        assert_eq!(is_zero_u64(u64::MAX).as_u8(), 0);
        assert_eq!(select_u32(Choice::from_bool(true), 7, 9), 7);
        assert_eq!(select_u32(Choice::from_bool(false), 7, 9), 9);
        assert_eq!(select_u64(Choice::from_bool(true), 7, 9), 7);
        assert_eq!(select_u64(Choice::from_bool(false), 7, 9), 9);
    }

    #[test]
    fn wide_integer_edges_are_covered() {
        for value in [0_u16, 1, 2, 0x8000, u16::MAX] {
            assert_eq!(is_zero_u16(value).as_u8(), u8::from(value == 0));
        }
        assert_eq!(eq_u16(1, 1).as_u8(), 1);
        assert_eq!(eq_u16(1, 0).as_u8(), 0);
        assert_eq!(eq_u16(1, 2).as_u8(), 0);
        assert_eq!(select_u16(Choice::from_bool(true), u16::MAX, 0), u16::MAX);
        assert_eq!(select_u16(Choice::from_bool(false), u16::MAX, 0), 0);

        for value in [0_u128, 1, 2, 1 << 127, u128::MAX] {
            assert_eq!(is_zero_u128(value).as_u8(), u8::from(value == 0));
        }
        assert_eq!(eq_u128(1, 1).as_u8(), 1);
        assert_eq!(eq_u128(1, 0).as_u8(), 0);
        assert_eq!(eq_u128(1, 2).as_u8(), 0);
        assert_eq!(
            select_u128(Choice::from_bool(true), u128::MAX, 0),
            u128::MAX
        );
        assert_eq!(select_u128(Choice::from_bool(false), u128::MAX, 0), 0);
    }
}
