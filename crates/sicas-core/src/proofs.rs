//! Kani proof harnesses. This module is compiled only under `cfg(kani)`.

use crate::{Choice, Masked, ct};

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
