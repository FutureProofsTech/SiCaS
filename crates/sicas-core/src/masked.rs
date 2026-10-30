//! First-order Boolean masking with two shares.

use core::fmt;

use crate::{Choice, Secret, ct};

/// A value represented by two XOR shares.
///
/// For a masked value `m`, `share0 ^ share1` is the original value. The two
/// shares must not be combined except at an explicit declassification boundary.
///
/// Equality and ordering are intentionally unavailable. Use
/// [`Masked::ct_eq`] for constant-time equality.
///
/// # Zeroization
///
/// This type does not guarantee that shares are erased from memory when
/// dropped. Production deployments must use a platform adapter that provides
/// verified zeroization.
///
/// # Example
///
/// ```
/// use sicas_core::Masked;
///
/// let value = 0x1234_5678_u32;
/// let masked = Masked::mask(value, 0xa5a5_a5a5_u32);
///
/// assert_eq!(*masked.unmask().expose_secret(), value);
/// ```
///
/// Equality is intentionally unavailable:
///
/// ```compile_fail
/// use sicas_core::Masked;
///
/// let a = Masked::from_parts(1, 2);
/// let b = Masked::from_parts(1, 2);
/// assert_eq!(a, b);
/// ```
#[must_use]
pub struct Masked<T> {
    share0: T,
    share1: T,
}

impl<T> fmt::Debug for Masked<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Masked<redacted>")
    }
}

impl Masked<u32> {
    /// Creates a masked value from two existing shares.
    ///
    /// The caller is responsible for the shares being independent and for not
    /// using the result as a fresh mask without re-randomization.
    pub const fn from_parts(share0: u32, share1: u32) -> Self {
        Self { share0, share1 }
    }

    /// Masks `value` with `mask`.
    ///
    /// # Security
    ///
    /// `mask` must be uniformly random and independent of `value`.
    pub const fn mask(value: u32, mask: u32) -> Self {
        Self {
            share0: value ^ mask,
            share1: mask,
        }
    }

    /// Declassifies the masked value.
    ///
    /// This is a declassification boundary. The caller must justify it in the
    /// surrounding leakage contract.
    pub const fn unmask(&self) -> Secret<u32> {
        Secret::new(self.share0 ^ self.share1)
    }

    /// XORs two masked values.
    pub const fn xor(&self, other: &Self) -> Self {
        Self {
            share0: self.share0 ^ other.share0,
            share1: self.share1 ^ other.share1,
        }
    }

    /// Returns the bitwise complement of the masked value.
    pub const fn complement(&self) -> Self {
        Self {
            share0: !self.share0,
            share1: self.share1,
        }
    }

    /// Rotates the masked value left by `amount` bits.
    pub const fn rotate_left(&self, amount: u32) -> Self {
        Self {
            share0: self.share0.rotate_left(amount),
            share1: self.share1.rotate_left(amount),
        }
    }

    /// Rotates the masked value right by `amount` bits.
    pub const fn rotate_right(&self, amount: u32) -> Self {
        Self {
            share0: self.share0.rotate_right(amount),
            share1: self.share1.rotate_right(amount),
        }
    }

    /// Shifts the masked value left by `amount` bits.
    ///
    /// Amounts greater than or equal to 32 produce zero.
    pub const fn shift_left(&self, amount: u32) -> Self {
        let share0 = match self.share0.checked_shl(amount) {
            Some(value) => value,
            None => 0,
        };
        let share1 = match self.share1.checked_shl(amount) {
            Some(value) => value,
            None => 0,
        };
        Self { share0, share1 }
    }

    /// Shifts the masked value right by `amount` bits.
    ///
    /// Amounts greater than or equal to 32 produce zero.
    pub const fn shift_right(&self, amount: u32) -> Self {
        let share0 = match self.share0.checked_shr(amount) {
            Some(value) => value,
            None => 0,
        };
        let share1 = match self.share1.checked_shr(amount) {
            Some(value) => value,
            None => 0,
        };
        Self { share0, share1 }
    }

    /// Computes the bitwise AND of two masked values.
    ///
    /// # Security
    ///
    /// `fresh_mask` must be uniformly random, independent of both operands, and
    /// must not be reused.
    pub const fn and(&self, other: &Self, fresh_mask: u32) -> Self {
        let share0 = (self.share0 & other.share0) ^ fresh_mask;
        let share1 = (self.share0 & other.share1)
            ^ (self.share1 & other.share0)
            ^ (self.share1 & other.share1)
            ^ fresh_mask;
        Self { share0, share1 }
    }

    /// Compares two masked values in constant time.
    pub const fn ct_eq(&self, other: &Self) -> Choice {
        ct::eq_u32(self.share0 ^ other.share0, self.share1 ^ other.share1)
    }

    /// Selects `a` when `choice` is true, otherwise `b`, in constant time.
    ///
    /// `choice` is a public selection bit. The shares are selected
    /// independently, so the result remains masked.
    pub fn select(choice: Choice, a: &Self, b: &Self) -> Self {
        Self {
            share0: ct::select_u32(choice, a.share0, b.share0),
            share1: ct::select_u32(choice, a.share1, b.share1),
        }
    }
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

    struct TestRng(u64);

    impl TestRng {
        const fn new(seed: u64) -> Self {
            Self(seed)
        }

        fn next_u32(&mut self) -> u32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            u32::try_from(self.0 >> 32).unwrap()
        }
    }

    #[test]
    fn mask_and_unmask_round_trip() {
        let mut rng = TestRng::new(0x5eed);

        for _ in 0..10_000 {
            let value = rng.next_u32();
            let mask = rng.next_u32();
            let masked = Masked::mask(value, mask);
            assert_eq!(*masked.unmask().expose_secret(), value);
        }
    }

    #[test]
    fn linear_operations_match_unmasked_reference() {
        let mut rng = TestRng::new(0x00c0_ffee);

        for _ in 0..10_000 {
            let a = rng.next_u32();
            let b = rng.next_u32();
            let mask_a = rng.next_u32();
            let mask_b = rng.next_u32();
            let fresh = rng.next_u32();
            let amount = rng.next_u32() % 32;

            let masked_a = Masked::mask(a, mask_a);
            let masked_b = Masked::mask(b, mask_b);

            assert_eq!(*masked_a.xor(&masked_b).unmask().expose_secret(), a ^ b);
            assert_eq!(*masked_a.complement().unmask().expose_secret(), !a);
            assert_eq!(
                *masked_a.rotate_left(amount).unmask().expose_secret(),
                a.rotate_left(amount)
            );
            assert_eq!(
                *masked_a.rotate_right(amount).unmask().expose_secret(),
                a.rotate_right(amount)
            );
            assert_eq!(
                *masked_a.shift_left(amount).unmask().expose_secret(),
                a.checked_shl(amount).unwrap_or(0)
            );
            assert_eq!(
                *masked_a.shift_right(amount).unmask().expose_secret(),
                a.checked_shr(amount).unwrap_or(0)
            );
            assert_eq!(
                *masked_a.and(&masked_b, fresh).unmask().expose_secret(),
                a & b
            );
        }
    }

    #[test]
    fn constant_time_equality_is_exact() {
        let value = 0x1234_5678_u32;
        let same = Masked::mask(value, 0xaaaa_aaaa);
        let different = Masked::mask(value ^ 1, 0x5555_5555);

        assert_eq!(same.ct_eq(&same).as_u8(), 1);
        assert_eq!(same.ct_eq(&different).as_u8(), 0);
    }

    #[test]
    fn selection_keeps_masking() {
        let a = Masked::mask(0x1111_1111, 0xaaaa_aaaa);
        let b = Masked::mask(0x2222_2222, 0x5555_5555);

        let selected_a = Masked::select(Choice::from_bool(true), &a, &b);
        let selected_b = Masked::select(Choice::from_bool(false), &a, &b);

        assert_eq!(*selected_a.unmask().expose_secret(), 0x1111_1111);
        assert_eq!(*selected_b.unmask().expose_secret(), 0x2222_2222);
    }

    #[test]
    fn debug_does_not_reveal_shares() {
        let masked = Masked::mask(0xdead_beef, 0x1234_5678);
        let rendered = std::format!("{masked:?}");

        assert_eq!(rendered, "Masked<redacted>");
        assert!(!rendered.contains("dead"));
    }
}
