//! Assembly inspection target for constant-time primitives.
//!
//! Build with:
//!
//! ```bash
//! cargo rustc -p sicas-core --release --example ct_asm -- --emit=asm
//! ```
//!
//! Then inspect the generated `.s` file in `target/release/examples/`.

#![allow(clippy::missing_const_for_fn)]

use std::hint::black_box;

use sicas_core::{Choice, EntropyError, EntropySource, Masked, ct};

struct FixedEntropy;

impl EntropySource for FixedEntropy {
    fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), EntropyError> {
        for byte in destination {
            *byte = 0x5a;
        }
        Ok(())
    }
}

#[inline(never)]
fn asm_eq_u32(left: u32, right: u32) -> u8 {
    ct::eq_u32(black_box(left), black_box(right)).as_u8()
}

#[inline(never)]
fn asm_select_u32(choice: bool, left: u32, right: u32) -> u32 {
    ct::select_u32(
        Choice::from_bool(black_box(choice)),
        black_box(left),
        black_box(right),
    )
}

#[inline(never)]
fn asm_is_zero_u32(value: u32) -> u8 {
    ct::is_zero_u32(black_box(value)).as_u8()
}

#[inline(never)]
fn asm_lookup_u8(index: u8) -> u8 {
    let table: [u8; 256] = core::array::from_fn(|value| u8::try_from(value).unwrap_or(0));
    ct::lookup_u8(black_box(&table), black_box(index))
}

#[inline(never)]
fn asm_masked_add(left: u32, right: u32) -> u32 {
    let masked_left = Masked::<u32>::mask(black_box(left), black_box(0xaaaa_aaaa));
    let masked_right = Masked::<u32>::mask(black_box(right), black_box(0x5555_5555));
    let mut entropy = FixedEntropy;
    masked_left
        .add_masked(&masked_right, &mut entropy)
        .map_or(0, |sum| *sum.unmask().expose_secret())
}

fn main() {
    let left = black_box(0x1234_5678_u32);
    let right = black_box(0x9abc_def0_u32);

    black_box(asm_eq_u32(left, right));
    black_box(asm_select_u32(true, left, right));
    black_box(asm_is_zero_u32(left));
    black_box(asm_lookup_u8(7));
    black_box(asm_masked_add(left, right));
}
