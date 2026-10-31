//! Local performance evidence for masked ChaCha20.
//!
//! This is a benchmark, not a security proof. It compares the masked reference
//! block against an unmasked reference implementation on the same machine.
//!
//! Run with:
//!
//! ```bash
//! cargo run -p sicas-reference --release --example benchmark
//! ```

#![allow(
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::missing_const_for_fn,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]

use std::hint::black_box;
use std::time::Instant;

use sicas_core::{EntropyError, EntropySource};
use sicas_reference::chacha20::{BATCH_LANES, MaskedChaCha20};

const CONSTANTS: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];
const BLOCKS: usize = 4_096;
const BATCHES: usize = 64;
const WARMUP: usize = 64;

struct XorShift(u64);

impl EntropySource for XorShift {
    fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), EntropyError> {
        for byte in destination {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            *byte = (self.0 & 0xff) as u8;
        }
        Ok(())
    }

    fn mask_u32(&mut self) -> Result<u32, EntropyError> {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        Ok(self.0 as u32)
    }

    fn mask_u64(&mut self) -> Result<u64, EntropyError> {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        Ok(self.0)
    }
}

fn word_from_bytes(bytes: [u8; 4]) -> u32 {
    u32::from_le_bytes(bytes)
}

fn quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(16);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(12);
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(8);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(7);
}

fn unmasked_block(key: &[u8; 32], nonce: &[u8; 12], counter: u32) -> [u8; 64] {
    let mut state = [0_u32; 16];
    state[..4].copy_from_slice(&CONSTANTS);
    for (index, chunk) in key.as_chunks::<4>().0.iter().enumerate() {
        state[4 + index] = word_from_bytes(*chunk);
    }
    state[12] = counter;
    for (index, chunk) in nonce.as_chunks::<4>().0.iter().enumerate() {
        state[13 + index] = word_from_bytes(*chunk);
    }

    let initial = state;
    for _ in 0..10 {
        quarter_round(&mut state, 0, 4, 8, 12);
        quarter_round(&mut state, 1, 5, 9, 13);
        quarter_round(&mut state, 2, 6, 10, 14);
        quarter_round(&mut state, 3, 7, 11, 15);
        quarter_round(&mut state, 0, 5, 10, 15);
        quarter_round(&mut state, 1, 6, 11, 12);
        quarter_round(&mut state, 2, 7, 8, 13);
        quarter_round(&mut state, 3, 4, 9, 14);
    }

    for (word, initial_word) in state.iter_mut().zip(initial) {
        *word = word.wrapping_add(initial_word);
    }

    let mut output = [0_u8; 64];
    for (chunk, word) in output.as_chunks_mut::<4>().0.iter_mut().zip(state) {
        chunk.copy_from_slice(&word.to_le_bytes());
    }
    output
}

fn nanos_per_byte(elapsed_ns: f64, blocks: usize) -> f64 {
    elapsed_ns / (blocks as f64 * 64.0)
}

fn main() {
    let key = [0x42_u8; 32];
    let nonce = [0x24_u8; 12];
    let mut entropy = XorShift(0x1234_5678_9abc_def0);

    for _ in 0..WARMUP {
        let mut cipher = MaskedChaCha20::new(&key, &nonce, 0, &mut entropy).unwrap();
        black_box(cipher.keystream_block(&mut entropy).unwrap());
    }
    for counter in 0..WARMUP as u32 {
        black_box(unmasked_block(&key, &nonce, counter));
    }

    let mut cipher = MaskedChaCha20::new(&key, &nonce, 0, &mut entropy).unwrap();
    let masked_start = Instant::now();
    for _ in 0..BLOCKS {
        black_box(cipher.keystream_block(&mut entropy).unwrap());
    }
    let masked_ns = masked_start.elapsed().as_secs_f64() * 1_000_000_000.0;

    let unmasked_start = Instant::now();
    for counter in 0..BLOCKS as u32 {
        black_box(unmasked_block(&key, &nonce, counter));
    }
    let unmasked_ns = unmasked_start.elapsed().as_secs_f64() * 1_000_000_000.0;

    let masked_per_byte = nanos_per_byte(masked_ns, BLOCKS);
    let unmasked_per_byte = nanos_per_byte(unmasked_ns, BLOCKS);
    let ratio = masked_per_byte / unmasked_per_byte;

    let mut batch_cipher = MaskedChaCha20::new(&key, &nonce, 0, &mut entropy).unwrap();
    for _ in 0..2 {
        black_box(batch_cipher.keystream_blocks(&mut entropy).unwrap());
    }
    let batch_start = Instant::now();
    for _ in 0..BATCHES {
        black_box(batch_cipher.keystream_blocks(&mut entropy).unwrap());
    }
    let batch_ns = batch_start.elapsed().as_secs_f64() * 1_000_000_000.0;
    let batch_blocks = BATCHES * BATCH_LANES;
    let batch_per_byte = nanos_per_byte(batch_ns, batch_blocks);
    let batch_ratio = batch_per_byte / unmasked_per_byte;

    println!("masked_blocks={BLOCKS}");
    println!("masked_ns_per_byte={masked_per_byte:.6}");
    println!("unmasked_ns_per_byte={unmasked_per_byte:.6}");
    println!("scalar_ratio={ratio:.3}");
    println!("batch_blocks={batch_blocks}");
    println!("batch_ns_per_byte={batch_per_byte:.6}");
    println!("batch_ratio={batch_ratio:.3}");
    println!("target_ratio<=5 hard_ceiling<=10");

    if batch_ratio > 10.0 {
        eprintln!("bitsliced masked ChaCha20 exceeded the hard 10x overhead ceiling");
        std::process::exit(1);
    }
}
