#![no_main]

use libfuzzer_sys::fuzz_target;
use sicas_core::{EntropyError, EntropySource};
use sicas_reference::chacha20::{BATCH_LANES, MaskedChaCha20};

const CONSTANTS: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];

struct FuzzEntropy(u64);

impl FuzzEntropy {
    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        u32::try_from(self.0 >> 32).unwrap()
    }
}

impl EntropySource for FuzzEntropy {
    fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), EntropyError> {
        for byte in destination {
            *byte = u8::try_from(self.next_u32() & 0xff).unwrap();
        }
        Ok(())
    }
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

fn reference_block(key: &[u8; 32], nonce: &[u8; 12], counter: u32) -> [u8; 64] {
    let mut state = [0_u32; 16];
    state[..4].copy_from_slice(&CONSTANTS);
    for (index, chunk) in key.chunks_exact(4).enumerate() {
        state[4 + index] = u32::from_le_bytes(chunk.try_into().unwrap());
    }
    state[12] = counter;
    for (index, chunk) in nonce.chunks_exact(4).enumerate() {
        state[13 + index] = u32::from_le_bytes(chunk.try_into().unwrap());
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
    for (chunk, word) in output.chunks_exact_mut(4).zip(state) {
        chunk.copy_from_slice(&word.to_le_bytes());
    }
    output
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 48 {
        return;
    }

    let mut key = [0_u8; 32];
    key.copy_from_slice(&data[..32]);
    let mut nonce = [0_u8; 12];
    nonce.copy_from_slice(&data[32..44]);
    let counter = u32::from_le_bytes([data[44], data[45], data[46], data[47]]);

    let seed = u64::from_le_bytes([
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ]);
    let mut entropy = FuzzEntropy(seed);
    let mut cipher = MaskedChaCha20::new(&key, &nonce, counter, &mut entropy).unwrap();
    let blocks = cipher.keystream_blocks(&mut entropy).unwrap();

    for (lane, block) in blocks.iter().enumerate() {
        let expected = reference_block(
            &key,
            &nonce,
            counter.wrapping_add(u32::try_from(lane).unwrap()),
        );
        assert_eq!(*block, expected);
    }

    debug_assert_eq!(blocks.len(), BATCH_LANES);
});