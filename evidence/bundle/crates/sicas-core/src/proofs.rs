//! Kani proof harnesses. This module is compiled only under `cfg(kani)`.

use crate::{Choice, EntropyError, EntropySource, Masked, ct};

#[kani::proof]
fn is_zero_u8_matches_reference() {
    let value: u8 = kani::any();
    let expected = if value == 0 { 1 } else { 0 };
    kani::assert(
        ct::is_zero_u8(value).as_u8() == expected,
        "is_zero_u8 must match the reference",
    );
}

#[kani::proof]
fn eq_u32_matches_reference() {
    let a: u32 = kani::any();
    let b: u32 = kani::any();
    let expected = if a == b { 1 } else { 0 };
    kani::assert(
        ct::eq_u32(a, b).as_u8() == expected,
        "eq_u32 must match the reference",
    );
}

#[kani::proof]
fn eq_u128_matches_reference() {
    let a: u128 = kani::any();
    let b: u128 = kani::any();
    let expected = if a == b { 1 } else { 0 };
    kani::assert(
        ct::eq_u128(a, b).as_u8() == expected,
        "eq_u128 must match the reference",
    );
}

#[kani::proof]
fn select_u32_matches_reference() {
    let choice = Choice::from_bool(kani::any());
    let a: u32 = kani::any();
    let b: u32 = kani::any();
    let result = ct::select_u32(choice, a, b);

    if choice.as_u8() == 1 {
        kani::assert(result == a, "select_u32 must select a when true");
    } else {
        kani::assert(result == b, "select_u32 must select b when false");
    }
}

#[kani::proof]
fn select_u128_matches_reference() {
    let choice = Choice::from_bool(kani::any());
    let a: u128 = kani::any();
    let b: u128 = kani::any();
    let result = ct::select_u128(choice, a, b);

    if choice.as_u8() == 1 {
        kani::assert(result == a, "select_u128 must select a when true");
    } else {
        kani::assert(result == b, "select_u128 must select b when false");
    }
}

#[kani::proof]
fn lookup_u8_matches_reference() {
    let table: [u8; 256] = core::array::from_fn(|index| u8::try_from(index).unwrap_or(0));
    let index: u8 = kani::any();
    let expected = table[usize::from(index)];
    kani::assert(
        ct::lookup_u8(&table, index) == expected,
        "lookup_u8 must return the selected table entry",
    );
}

#[kani::proof]
fn masked_u32_round_trip() {
    let value: u32 = kani::any();
    let mask: u32 = kani::any();
    let masked = Masked::<u32>::mask(value, mask);
    kani::assert(
        *masked.unmask().expose_secret() == value,
        "masking must preserve the value",
    );
}

#[kani::proof]
fn masked_u8_share0_covers_all_values() {
    let secret: u8 = kani::any();
    let target: u8 = kani::any();
    let mask = secret ^ target;
    let masked = Masked::<u8>::mask(secret, mask);

    kani::assert(
        *masked.shares().0 == target,
        "the first share must cover every value for every secret",
    );
}

#[kani::proof]
fn masked_u32_share0_covers_all_values() {
    let secret: u32 = kani::any();
    let target: u32 = kani::any();
    let mask = secret ^ target;
    let masked = Masked::<u32>::mask(secret, mask);

    kani::assert(
        *masked.shares().0 == target,
        "the first share must cover every value for every secret",
    );
}

#[kani::proof]
fn masked_u8_and_share0_covers_all_values() {
    let a: u8 = kani::any();
    let b: u8 = kani::any();
    let mask_a: u8 = kani::any();
    let mask_b: u8 = kani::any();
    let target: u8 = kani::any();

    let masked_a = Masked::<u8>::mask(a, mask_a);
    let masked_b = Masked::<u8>::mask(b, mask_b);
    let base = *masked_a.shares().0 & *masked_b.shares().0;
    let fresh_mask = base ^ target;
    let result = masked_a.and(&masked_b, fresh_mask);

    kani::assert(
        *result.shares().0 == target,
        "the masked-AND first share must cover every value",
    );
}

#[kani::proof]
fn masked_u32_shares_reconstruct_value() {
    let value: u32 = kani::any();
    let mask: u32 = kani::any();
    let masked = Masked::<u32>::mask(value, mask);
    let (share0, share1) = masked.shares();

    kani::assert(
        *share0 ^ *share1 == value,
        "the two shares must reconstruct the value",
    );
}

#[kani::proof]
fn masked_u128_round_trip() {
    let value: u128 = kani::any();
    let mask: u128 = kani::any();
    let masked = Masked::<u128>::mask(value, mask);
    kani::assert(
        *masked.unmask().expose_secret() == value,
        "masking must preserve the value",
    );
}

#[kani::proof]
fn masked_u32_and_matches_reference() {
    let a: u32 = kani::any();
    let b: u32 = kani::any();
    let mask_a: u32 = kani::any();
    let mask_b: u32 = kani::any();
    let fresh_mask: u32 = kani::any();

    let masked_a = Masked::<u32>::mask(a, mask_a);
    let masked_b = Masked::<u32>::mask(b, mask_b);
    let result = masked_a.and(&masked_b, fresh_mask);

    kani::assert(
        *result.unmask().expose_secret() == (a & b),
        "masked AND must match the reference",
    );
}

#[kani::proof]
fn masked_u128_and_matches_reference() {
    let a: u128 = kani::any();
    let b: u128 = kani::any();
    let mask_a: u128 = kani::any();
    let mask_b: u128 = kani::any();
    let fresh_mask: u128 = kani::any();

    let masked_a = Masked::<u128>::mask(a, mask_a);
    let masked_b = Masked::<u128>::mask(b, mask_b);
    let result = masked_a.and(&masked_b, fresh_mask);

    kani::assert(
        *result.unmask().expose_secret() == (a & b),
        "masked AND must match the reference",
    );
}

#[kani::proof]
fn masked_u32_ct_eq_matches_reference() {
    let a: u32 = kani::any();
    let b: u32 = kani::any();
    let mask_a: u32 = kani::any();
    let mask_b: u32 = kani::any();

    let masked_a = Masked::<u32>::mask(a, mask_a);
    let masked_b = Masked::<u32>::mask(b, mask_b);
    let expected = if a == b { 1 } else { 0 };

    kani::assert(
        masked_a.ct_eq(&masked_b).as_u8() == expected,
        "masked equality must match the reference",
    );
}

#[kani::proof]
fn masked_u32_select_matches_reference() {
    let choice = Choice::from_bool(kani::any());
    let a: u32 = kani::any();
    let b: u32 = kani::any();
    let mask_a: u32 = kani::any();
    let mask_b: u32 = kani::any();

    let masked_a = Masked::<u32>::mask(a, mask_a);
    let masked_b = Masked::<u32>::mask(b, mask_b);
    let selected = Masked::<u32>::select(choice, &masked_a, &masked_b);

    if choice.as_u8() == 1 {
        kani::assert(
            *selected.unmask().expose_secret() == a,
            "masked select must select a when true",
        );
    } else {
        kani::assert(
            *selected.unmask().expose_secret() == b,
            "masked select must select b when false",
        );
    }
}

struct KaniEntropy(u32);

impl EntropySource for KaniEntropy {
    fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), EntropyError> {
        let bytes = self.0.to_le_bytes();
        for (index, byte) in destination.iter_mut().enumerate() {
            *byte = bytes[index % 4];
        }
        self.0 = self.0.wrapping_add(0x9e37_79b9);
        Ok(())
    }
}

#[kani::proof]
fn masked_u32_refresh_preserves_value() {
    let value: u32 = kani::any();
    let mask: u32 = kani::any();
    let fresh_mask: u32 = kani::any();
    let masked = Masked::<u32>::mask(value, mask);
    let refreshed = masked.refresh(fresh_mask);

    kani::assert(
        *refreshed.unmask().expose_secret() == value,
        "mask refresh must preserve the value",
    );
}

#[kani::proof]
fn masked_u32_add_matches_reference() {
    let a: u32 = kani::any();
    let b: u32 = kani::any();
    let mask_a: u32 = kani::any();
    let mask_b: u32 = kani::any();
    let masked_a = Masked::<u32>::mask(a, mask_a);
    let masked_b = Masked::<u32>::mask(b, mask_b);
    let mut entropy = KaniEntropy(0x1234_5678);
    let sum = masked_a.add_masked(&masked_b, &mut entropy).unwrap();

    kani::assert(
        *sum.unmask().expose_secret() == a.wrapping_add(b),
        "masked addition must match wrapping addition",
    );
}

#[kani::proof]
fn masked_u32_shift_matches_reference() {
    let value: u32 = kani::any();
    let mask: u32 = kani::any();
    let amount: u32 = kani::any();
    let masked = Masked::<u32>::mask(value, mask);

    let expected_left = value.checked_shl(amount).unwrap_or(0);
    let expected_right = value.checked_shr(amount).unwrap_or(0);

    kani::assert(
        *masked.shift_left(amount).unmask().expose_secret() == expected_left,
        "masked left shift must match the reference",
    );
    kani::assert(
        *masked.shift_right(amount).unmask().expose_secret() == expected_right,
        "masked right shift must match the reference",
    );
}

macro_rules! linear_operation_proof {
    ($name:ident, $type:ty) => {
        #[kani::proof]
        fn $name() {
            let a: $type = kani::any();
            let b: $type = kani::any();
            let mask_a: $type = kani::any();
            let mask_b: $type = kani::any();
            let amount: u32 = kani::any();
            let masked_a = Masked::<$type>::mask(a, mask_a);
            let masked_b = Masked::<$type>::mask(b, mask_b);

            kani::assert(
                *masked_a.xor(&masked_b).unmask().expose_secret() == a ^ b,
                "masked XOR must match the reference",
            );
            kani::assert(
                *masked_a.complement().unmask().expose_secret() == !a,
                "masked complement must match the reference",
            );
            kani::assert(
                *masked_a.rotate_left(amount).unmask().expose_secret() == a.rotate_left(amount),
                "masked left rotation must match the reference",
            );
            kani::assert(
                *masked_a.rotate_right(amount).unmask().expose_secret() == a.rotate_right(amount),
                "masked right rotation must match the reference",
            );
            kani::assert(
                *masked_a.shift_left(amount).unmask().expose_secret()
                    == a.checked_shl(amount).unwrap_or(0),
                "masked left shift must match the reference",
            );
            kani::assert(
                *masked_a.shift_right(amount).unmask().expose_secret()
                    == a.checked_shr(amount).unwrap_or(0),
                "masked right shift must match the reference",
            );
        }
    };
}

macro_rules! logic_operation_proof {
    ($name:ident, $type:ty) => {
        #[kani::proof]
        fn $name() {
            let choice = Choice::from_bool(kani::any());
            let a: $type = kani::any();
            let b: $type = kani::any();
            let mask_a: $type = kani::any();
            let mask_b: $type = kani::any();
            let fresh_mask: $type = kani::any();
            let masked_a = Masked::<$type>::mask(a, mask_a);
            let masked_b = Masked::<$type>::mask(b, mask_b);

            kani::assert(
                *masked_a.and(&masked_b, fresh_mask).unmask().expose_secret() == a & b,
                "masked AND must match the reference",
            );
            kani::assert(
                masked_a.ct_eq(&masked_b).as_u8() == u8::from(a == b),
                "masked equality must match the reference",
            );
            kani::assert(
                *masked_a.refresh(fresh_mask).unmask().expose_secret() == a,
                "mask refresh must preserve the value",
            );

            let selected = Masked::<$type>::select(choice, &masked_a, &masked_b);
            if choice.as_u8() == 1 {
                kani::assert(
                    *selected.unmask().expose_secret() == a,
                    "masked select must select a when true",
                );
            } else {
                kani::assert(
                    *selected.unmask().expose_secret() == b,
                    "masked select must select b when false",
                );
            }
        }
    };
}

linear_operation_proof!(masked_u8_linear_ops_match_reference, u8);
linear_operation_proof!(masked_u16_linear_ops_match_reference, u16);
linear_operation_proof!(masked_u32_linear_ops_match_reference, u32);
linear_operation_proof!(masked_u64_linear_ops_match_reference, u64);
linear_operation_proof!(masked_u128_linear_ops_match_reference, u128);

logic_operation_proof!(masked_u8_logic_ops_match_reference, u8);
logic_operation_proof!(masked_u16_logic_ops_match_reference, u16);
logic_operation_proof!(masked_u32_logic_ops_match_reference, u32);
logic_operation_proof!(masked_u64_logic_ops_match_reference, u64);
logic_operation_proof!(masked_u128_logic_ops_match_reference, u128);

#[kani::proof]
fn masked_u32_bit_matches_reference() {
    let value: u32 = kani::any();
    let mask: u32 = kani::any();
    let index: u32 = kani::any();
    let masked = Masked::<u32>::mask(value, mask);
    let expected = value & 1_u32.checked_shl(index).unwrap_or(0);

    kani::assert(
        *masked.bit(index).unmask().expose_secret() == expected,
        "masked bit extraction must match the reference",
    );
}
