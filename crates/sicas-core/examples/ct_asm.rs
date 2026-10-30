//! Assembly inspection target for constant-time primitives.
//!
//! Build with:
//!
//! ```bash
//! cargo rustc -p sicas-core --release --example ct_asm -- --emit=asm
//! ```
//!
//! Then inspect the generated `.s` file in `target/release/examples/`.

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

fn main() {
    let left = black_box(0x1234_5678_u32);
    let right = black_box(0x9abc_def0_u32);
    let choice = Choice::from_bool(black_box(true));
    let table: [u8; 256] = core::array::from_fn(|index| u8::try_from(index).unwrap_or(0));

    let _ = black_box(ct::eq_u32(left, right));
    let _ = black_box(ct::select_u32(choice, left, right));
    let _ = black_box(ct::lookup_u8(&table, black_box(7_u8)));

    let masked_left = Masked::<u32>::mask(left, black_box(0xaaaa_aaaa));
    let masked_right = Masked::<u32>::mask(right, black_box(0x5555_5555));
    let mut entropy = FixedEntropy;
    black_box(masked_left.add_masked(&masked_right, &mut entropy).ok());
}
