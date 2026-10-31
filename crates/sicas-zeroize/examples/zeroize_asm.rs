//! Assembly inspection target for the zeroization boundary.
//!
//! Build with:
//!
//! ```bash
//! cargo rustc -p sicas-zeroize --release --example zeroize_asm -- --emit=asm
//! ```

use std::hint::black_box;

use sicas_zeroize::{ZeroizingBytes, zeroize_bytes};

#[inline(never)]
fn asm_zeroize(buffer: &mut [u8; 64]) {
    zeroize_bytes(buffer);
}

#[inline(never)]
fn asm_drop() {
    let mut buffer = ZeroizingBytes::new([0xaa_u8; 64]);
    black_box(&mut buffer);
}

fn main() {
    let mut buffer = [0xaa_u8; 64];
    asm_zeroize(&mut buffer);
    asm_drop();
    black_box(&buffer);
}
