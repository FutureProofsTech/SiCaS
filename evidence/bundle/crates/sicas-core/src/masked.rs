//! First-order Boolean masking with two shares.

use core::fmt;

use crate::{Choice, EntropyError, EntropySource, Secret, ct};

/// A value represented by two XOR shares.
///
/// For a masked value `m`, `share0 ^ share1` is the original value. The two
/// shares must not be combined except at an explicit declassification boundary.
///
/// Equality and ordering are intentionally unavailable. Use
/// [`Masked::ct_eq`] for constant-time equality. Cloning a masked value
/// duplicates its shares but does not re-randomize them.
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
/// let masked = Masked::<u32>::mask(value, 0xa5a5_a5a5_u32);
///
/// assert_eq!(*masked.unmask().expose_secret(), value);
/// ```
///
/// Equality is intentionally unavailable:
///
/// ```compile_fail
/// use sicas_core::Masked;
///
/// let a = Masked::<u32>::from_parts(1, 2);
/// let b = Masked::<u32>::from_parts(1, 2);
/// assert_eq!(a, b);
/// ```
#[derive(Clone)]
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

macro_rules! impl_masked {
    ($type:ty, $eq:path, $select:path) => {
        impl Masked<$type> {
            /// Creates a masked value from two existing shares.
            ///
            /// The caller is responsible for the shares being independent and
            /// for not using the result as a fresh mask without
            /// re-randomization.
            #[inline]
            pub const fn from_parts(share0: $type, share1: $type) -> Self {
                Self { share0, share1 }
            }

            /// Masks `value` with `mask`.
            ///
            /// # Security
            ///
            /// `mask` must be uniformly random and independent of `value`.
            #[inline]
            pub const fn mask(value: $type, mask: $type) -> Self {
                Self {
                    share0: value ^ mask,
                    share1: mask,
                }
            }

            /// Returns the two shares.
            ///
            /// # Security
            ///
            /// Raw shares are sensitive material. Use them only inside an
            /// explicit leakage-analysis or composition boundary. Never branch
            /// on, log, or publish individual shares.
            #[inline]
            pub const fn shares(&self) -> (&$type, &$type) {
                (&self.share0, &self.share1)
            }

            /// Declassifies the masked value.
            ///
            /// This is a declassification boundary. The caller must justify it
            /// in the surrounding leakage contract.
            #[inline]
            pub const fn unmask(&self) -> Secret<$type> {
                Secret::new(self.share0 ^ self.share1)
            }

            /// Re-randomizes the shares with a fresh mask.
            ///
            /// The logical value is unchanged. `fresh_mask` must be independent
            /// of the current shares and must not be reused.
            #[inline]
            pub const fn refresh(&self, fresh_mask: $type) -> Self {
                Self {
                    share0: self.share0 ^ fresh_mask,
                    share1: self.share1 ^ fresh_mask,
                }
            }

            /// XORs two masked values.
            #[inline]
            pub const fn xor(&self, other: &Self) -> Self {
                Self {
                    share0: self.share0 ^ other.share0,
                    share1: self.share1 ^ other.share1,
                }
            }

            /// Returns the bitwise complement of the masked value.
            #[inline]
            pub const fn complement(&self) -> Self {
                Self {
                    share0: !self.share0,
                    share1: self.share1,
                }
            }

            /// Rotates the masked value left by `amount` bits.
            ///
            /// The amount is reduced modulo the bit width.
            #[inline]
            pub const fn rotate_left(&self, amount: u32) -> Self {
                Self {
                    share0: self.share0.rotate_left(amount),
                    share1: self.share1.rotate_left(amount),
                }
            }

            /// Rotates the masked value right by `amount` bits.
            ///
            /// The amount is reduced modulo the bit width.
            #[inline]
            pub const fn rotate_right(&self, amount: u32) -> Self {
                Self {
                    share0: self.share0.rotate_right(amount),
                    share1: self.share1.rotate_right(amount),
                }
            }

            /// Shifts the masked value left by `amount` bits.
            ///
            /// Amounts greater than or equal to the bit width produce zero.
            #[inline]
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
            /// Amounts greater than or equal to the bit width produce zero.
            #[inline]
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
            /// `fresh_mask` must be uniformly random, independent of both
            /// operands, and must not be reused.
            #[inline]
            pub const fn and(&self, other: &Self, fresh_mask: $type) -> Self {
                let share0 = (self.share0 & other.share0) ^ fresh_mask;
                let share1 = (self.share0 & other.share1)
                    ^ (self.share1 & other.share0)
                    ^ (self.share1 & other.share1)
                    ^ fresh_mask;
                Self { share0, share1 }
            }

            /// Compares two masked values in constant time.
            #[inline]
            pub const fn ct_eq(&self, other: &Self) -> Choice {
                $eq(self.share0 ^ other.share0, self.share1 ^ other.share1)
            }

            /// Selects `a` when `choice` is true, otherwise `b`, in constant
            /// time.
            ///
            /// `choice` is a public selection bit. The shares are selected
            /// independently, so the result remains masked.
            #[inline]
            pub const fn select(choice: Choice, a: &Self, b: &Self) -> Self {
                Self {
                    share0: $select(choice, a.share0, b.share0),
                    share1: $select(choice, a.share1, b.share1),
                }
            }
        }
    };
}

impl_masked!(u8, ct::eq_u8, ct::select_u8);
impl_masked!(u16, ct::eq_u16, ct::select_u16);
impl_masked!(u32, ct::eq_u32, ct::select_u32);
impl_masked!(u64, ct::eq_u64, ct::select_u64);
impl_masked!(u128, ct::eq_u128, ct::select_u128);

impl Masked<u32> {
    /// Extracts one bit as a masked value.
    ///
    /// Bits outside `index` are zero. An index of 32 or greater returns zero.
    #[inline]
    pub const fn bit(&self, index: u32) -> Self {
        let mask = match 1_u32.checked_shl(index) {
            Some(value) => value,
            None => 0,
        };
        Self {
            share0: self.share0 & mask,
            share1: self.share1 & mask,
        }
    }

    /// Adds two masked `u32` values with a fresh mask for every masked AND.
    ///
    /// This is a ripple-carry adder. The result remains masked; unmask it only
    /// at an explicit declassification boundary.
    ///
    /// # Security
    ///
    /// `entropy` must provide fresh, independent masks. Reusing masks or using a
    /// predictable source invalidates the first-order masking assumption.
    ///
    /// # Errors
    ///
    /// Returns [`EntropyError::Unavailable`] when the entropy source fails.
    #[inline]
    pub fn add_masked<E: EntropySource>(
        &self,
        other: &Self,
        entropy: &mut E,
    ) -> Result<Self, EntropyError> {
        let mut result = Self::from_parts(0, 0);
        let mut carry = Self::from_parts(0, 0);

        for index in 0..32 {
            let a = self.bit(index);
            let b = other.bit(index);
            let sum = a.xor(&b).xor(&carry);
            result = result.xor(&sum);

            if index == 31 {
                break;
            }

            let first = a.and(&b, entropy.mask_u32()?);
            let second = a.and(&carry, entropy.mask_u32()?);
            let third = b.and(&carry, entropy.mask_u32()?);
            carry = first.xor(&second).xor(&third).shift_left(1);
        }

        Ok(result)
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

        fn next_u8(&mut self) -> u8 {
            u8::try_from(self.next_u32() & 0xff).unwrap()
        }
    }

    struct TestEntropy(TestRng);

    impl TestEntropy {
        const fn new(seed: u64) -> Self {
            Self(TestRng::new(seed))
        }
    }

    impl EntropySource for TestEntropy {
        fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), EntropyError> {
            for byte in destination {
                *byte = self.0.next_u8();
            }
            Ok(())
        }
    }

    struct FailingEntropy;

    impl EntropySource for FailingEntropy {
        fn fill_bytes(&mut self, _destination: &mut [u8]) -> Result<(), EntropyError> {
            Err(EntropyError::Unavailable)
        }
    }

    #[test]
    fn shares_reconstruct_the_value() {
        let value = 0x1234_5678_u32;
        let mask = 0xaaaa_aaaa_u32;
        let masked = Masked::<u32>::mask(value, mask);
        let (share0, share1) = masked.shares();

        assert_eq!(*share0 ^ *share1, value);
        assert_eq!(*share0, value ^ mask);
        assert_eq!(*share1, mask);
    }

    #[test]
    fn mask_and_unmask_round_trip() {
        let mut rng = TestRng::new(0x5eed);
        let rounds = if cfg!(miri) { 8 } else { 10_000 };

        for _ in 0..rounds {
            let value = rng.next_u32();
            let mask = rng.next_u32();
            let masked = Masked::<u32>::mask(value, mask);
            assert_eq!(*masked.unmask().expose_secret(), value);
        }
    }

    #[test]
    fn linear_operations_match_unmasked_reference() {
        let mut rng = TestRng::new(0x00c0_ffee);
        let rounds = if cfg!(miri) { 8 } else { 10_000 };

        for _ in 0..rounds {
            let a = rng.next_u32();
            let b = rng.next_u32();
            let mask_a = rng.next_u32();
            let mask_b = rng.next_u32();
            let fresh = rng.next_u32();
            let amount = rng.next_u32() % 32;

            let masked_a = Masked::<u32>::mask(a, mask_a);
            let masked_b = Masked::<u32>::mask(b, mask_b);

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
        let same = Masked::<u32>::mask(value, 0xaaaa_aaaa);
        let different = Masked::<u32>::mask(value ^ 1, 0x5555_5555);

        assert_eq!(same.ct_eq(&same).as_u8(), 1);
        assert_eq!(same.ct_eq(&different).as_u8(), 0);
    }

    #[test]
    fn selection_keeps_masking() {
        let a = Masked::<u32>::mask(0x1111_1111, 0xaaaa_aaaa);
        let b = Masked::<u32>::mask(0x2222_2222, 0x5555_5555);

        let selected_a = Masked::<u32>::select(Choice::from_bool(true), &a, &b);
        let selected_b = Masked::<u32>::select(Choice::from_bool(false), &a, &b);

        assert_eq!(*selected_a.unmask().expose_secret(), 0x1111_1111);
        assert_eq!(*selected_b.unmask().expose_secret(), 0x2222_2222);
    }

    #[test]
    fn bit_extraction_is_correct() {
        let value = 0x8000_0001_u32;
        let masked = Masked::<u32>::mask(value, 0xa5a5_a5a5);

        for index in 0..32 {
            let expected = value & (1_u32 << index);
            assert_eq!(*masked.bit(index).unmask().expose_secret(), expected);
        }
        assert_eq!(*masked.bit(32).unmask().expose_secret(), 0);
    }

    #[test]
    fn refresh_preserves_the_logical_value() {
        let value = 0x1234_5678_u32;
        let masked = Masked::<u32>::mask(value, 0xaaaa_aaaa);
        let refreshed = masked.refresh(0x5555_5555);

        assert_eq!(*refreshed.unmask().expose_secret(), value);
    }

    #[test]
    fn masked_addition_matches_reference() {
        let mut rng = TestRng::new(0xadd1_7e57);
        let rounds = if cfg!(miri) { 4 } else { 1_000 };

        for _ in 0..rounds {
            let a = rng.next_u32();
            let b = rng.next_u32();
            let masked_a = Masked::<u32>::mask(a, rng.next_u32());
            let masked_b = Masked::<u32>::mask(b, rng.next_u32());
            let mut entropy = TestEntropy::new(rng.next_u32().into());

            let sum = masked_a.add_masked(&masked_b, &mut entropy).unwrap();
            assert_eq!(*sum.unmask().expose_secret(), a.wrapping_add(b));
        }
    }

    #[test]
    fn masked_addition_fails_closed() {
        let a = Masked::<u32>::mask(1, 2);
        let b = Masked::<u32>::mask(3, 4);
        let mut entropy = FailingEntropy;

        assert!(matches!(
            a.add_masked(&b, &mut entropy),
            Err(EntropyError::Unavailable)
        ));
    }

    #[test]
    fn all_integer_widths_round_trip_and_operate() {
        macro_rules! check_width {
            ($type:ty, $a:expr, $b:expr, $mask_a:expr, $mask_b:expr, $fresh:expr) => {{
                let a: $type = $a;
                let b: $type = $b;
                let masked_a = Masked::<$type>::mask(a, $mask_a);
                let masked_b = Masked::<$type>::mask(b, $mask_b);

                assert_eq!(*masked_a.unmask().expose_secret(), a);
                assert_eq!(*masked_a.xor(&masked_b).unmask().expose_secret(), a ^ b);
                assert_eq!(
                    *masked_a.and(&masked_b, $fresh).unmask().expose_secret(),
                    a & b
                );
                assert_eq!(masked_a.ct_eq(&masked_a).as_u8(), 1);
                assert_eq!(masked_a.ct_eq(&masked_b).as_u8(), u8::from(a == b));

                let selected =
                    Masked::<$type>::select(Choice::from_bool(true), &masked_a, &masked_b);
                assert_eq!(*selected.unmask().expose_secret(), a);
            }};
        }

        check_width!(u8, 0x12, 0x34, 0xaa, 0x55, 0x0f);
        check_width!(u16, 0x1234, 0x5678, 0xaaaa, 0x5555, 0x0f0f);
        check_width!(
            u32,
            0x1234_5678,
            0x9abc_def0,
            0xaaaa_aaaa,
            0x5555_5555,
            0x0f0f_0f0f
        );
        check_width!(
            u64,
            0x1234_5678_9abc_def0,
            0x0fed_cba9_8765_4321,
            0xaaaa_aaaa_aaaa_aaaa,
            0x5555_5555_5555_5555,
            0x0f0f_0f0f_0f0f_0f0f
        );
        check_width!(
            u128,
            0x1234_5678_9abc_def0_1234_5678_9abc_def0,
            0x0fed_cba9_8765_4321_0fed_cba9_8765_4321,
            0xaaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa,
            0x5555_5555_5555_5555_5555_5555_5555_5555,
            0x0f0f_0f0f_0f0f_0f0f_0f0f_0f0f_0f0f_0f0f
        );
    }

    #[test]
    fn debug_does_not_reveal_shares() {
        let masked = Masked::<u32>::mask(0xdead_beef, 0x1234_5678);
        let rendered = std::format!("{masked:?}");

        assert_eq!(rendered, "Masked<redacted>");
        assert!(!rendered.contains("dead"));
    }
}
