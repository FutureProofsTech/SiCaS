//! First-order Boolean masked ChaCha20 reference integration.
//!
//! This is a validation target, not an audited production cipher. The keystream
//! returned by [`MaskedChaCha20::keystream_block`] is a declassification
//! boundary.

use core::fmt;

use sicas_core::{EntropyError, EntropySource, Masked};

const CONSTANTS: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];

/// A masked ChaCha20 state.
///
/// The working state and the initial state used by the final addition are
/// independently masked. This avoids feeding correlated shares into the masked
/// adder. Zeroization is not guaranteed; use a verified platform adapter for
/// production key storage.
#[must_use]
pub struct MaskedChaCha20 {
    state: [Masked<u32>; 16],
    counter: u32,
}

impl fmt::Debug for MaskedChaCha20 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MaskedChaCha20<redacted>")
    }
}

impl MaskedChaCha20 {
    /// Creates a masked ChaCha20 state from a 256-bit key, 96-bit nonce, and
    /// initial block counter.
    ///
    /// # Errors
    ///
    /// Returns [`EntropyError::Unavailable`] when the entropy source fails.
    pub fn new(
        key: &[u8; 32],
        nonce: &[u8; 12],
        counter: u32,
        entropy: &mut impl EntropySource,
    ) -> Result<Self, EntropyError> {
        let mut state: [Masked<u32>; 16] =
            core::array::from_fn(|_| Masked::<u32>::from_parts(0, 0));

        for (slot, constant) in state.iter_mut().take(4).zip(CONSTANTS) {
            *slot = Masked::<u32>::from_parts(constant, 0);
        }

        for (slot, chunk) in state
            .iter_mut()
            .skip(4)
            .take(8)
            .zip(key.as_chunks::<4>().0.iter())
        {
            *slot = Masked::<u32>::mask(word_from_bytes(*chunk), entropy.mask_u32()?);
        }

        if let Some(slot) = state.get_mut(12) {
            *slot = Masked::<u32>::mask(counter, entropy.mask_u32()?);
        }

        for (slot, chunk) in state
            .iter_mut()
            .skip(13)
            .take(3)
            .zip(nonce.as_chunks::<4>().0.iter())
        {
            *slot = Masked::<u32>::from_parts(word_from_bytes(*chunk), 0);
        }

        Ok(Self { state, counter })
    }

    /// Produces one 64-byte ChaCha20 keystream block and increments the counter.
    ///
    /// The returned bytes are declassified keystream. Callers must justify this
    /// boundary in their leakage contract.
    ///
    /// # Errors
    ///
    /// Returns [`EntropyError::Unavailable`] when the entropy source fails.
    pub fn keystream_block(
        &mut self,
        entropy: &mut impl EntropySource,
    ) -> Result<[u8; 64], EntropyError> {
        let mut working: [Masked<u32>; 16] =
            core::array::from_fn(|_| Masked::<u32>::from_parts(0, 0));
        let mut initial: [Masked<u32>; 16] =
            core::array::from_fn(|_| Masked::<u32>::from_parts(0, 0));
        for ((working_slot, initial_slot), state_slot) in working
            .iter_mut()
            .zip(initial.iter_mut())
            .zip(self.state.iter())
        {
            *working_slot = state_slot.refresh(entropy.mask_u32()?);
            *initial_slot = state_slot.refresh(entropy.mask_u32()?);
        }

        for _ in 0..10 {
            quarter_round(&mut working, 0, 4, 8, 12, entropy)?;
            quarter_round(&mut working, 1, 5, 9, 13, entropy)?;
            quarter_round(&mut working, 2, 6, 10, 14, entropy)?;
            quarter_round(&mut working, 3, 7, 11, 15, entropy)?;
            quarter_round(&mut working, 0, 5, 10, 15, entropy)?;
            quarter_round(&mut working, 1, 6, 11, 12, entropy)?;
            quarter_round(&mut working, 2, 7, 8, 13, entropy)?;
            quarter_round(&mut working, 3, 4, 9, 14, entropy)?;
        }

        for (slot, initial_word) in working.iter_mut().zip(&initial) {
            *slot = slot.add_masked(initial_word, entropy)?;
        }

        let mut output = [0_u8; 64];
        for (chunk, word) in output.as_chunks_mut::<4>().0.iter_mut().zip(working.iter()) {
            chunk.copy_from_slice(&word.unmask().expose_secret().to_le_bytes());
        }

        let state_mask = entropy.mask_u32()?;
        let next_counter = self.counter.wrapping_add(1);
        self.counter = next_counter;
        if let Some(slot) = self.state.get_mut(12) {
            *slot = Masked::<u32>::mask(next_counter, state_mask);
        }

        Ok(output)
    }
}

const fn word_from_bytes(bytes: [u8; 4]) -> u32 {
    u32::from_le_bytes(bytes)
}

#[allow(clippy::indexing_slicing)] // The indices are fixed quarter-round positions.
fn quarter_round(
    state: &mut [Masked<u32>; 16],
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    entropy: &mut impl EntropySource,
) -> Result<(), EntropyError> {
    let mut a_value = state[a].clone();
    let mut b_value = state[b].clone();
    let mut c_value = state[c].clone();
    let mut d_value = state[d].clone();

    a_value = a_value.add_masked(&b_value, entropy)?;
    d_value = d_value.xor(&a_value).rotate_left(16);
    c_value = c_value.add_masked(&d_value, entropy)?;
    b_value = b_value.xor(&c_value).rotate_left(12);
    a_value = a_value.add_masked(&b_value, entropy)?;
    d_value = d_value.xor(&a_value).rotate_left(8);
    c_value = c_value.add_masked(&d_value, entropy)?;
    b_value = b_value.xor(&c_value).rotate_left(7);

    state[a] = a_value;
    state[b] = b_value;
    state[c] = c_value;
    state[d] = d_value;
    Ok(())
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
    use std::vec::Vec;

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

    fn reference_quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
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
        for (index, chunk) in key.as_chunks::<4>().0.iter().enumerate() {
            state[4 + index] = word_from_bytes(*chunk);
        }
        state[12] = counter;
        for (index, chunk) in nonce.as_chunks::<4>().0.iter().enumerate() {
            state[13 + index] = word_from_bytes(*chunk);
        }

        let initial = state;
        for _ in 0..10 {
            reference_quarter_round(&mut state, 0, 4, 8, 12);
            reference_quarter_round(&mut state, 1, 5, 9, 13);
            reference_quarter_round(&mut state, 2, 6, 10, 14);
            reference_quarter_round(&mut state, 3, 7, 11, 15);
            reference_quarter_round(&mut state, 0, 5, 10, 15);
            reference_quarter_round(&mut state, 1, 6, 11, 12);
            reference_quarter_round(&mut state, 2, 7, 8, 13);
            reference_quarter_round(&mut state, 3, 4, 9, 14);
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

    #[test]
    fn rfc8439_keystream_block_is_correct() {
        let key: [u8; 32] = core::array::from_fn(|index| u8::try_from(index).unwrap());
        let nonce = [0, 0, 0, 9, 0, 0, 0, 0x4a, 0, 0, 0, 0];
        let mut entropy = TestEntropy::new(0x1234_5678);
        let mut cipher = MaskedChaCha20::new(&key, &nonce, 1, &mut entropy).unwrap();
        let block = cipher.keystream_block(&mut entropy).unwrap();

        let expected = [
            0x10, 0xf1, 0xe7, 0xe4, 0xd1, 0x3b, 0x59, 0x15, 0x50, 0x0f, 0xdd, 0x1f, 0xa3, 0x20,
            0x71, 0xc4, 0xc7, 0xd1, 0xf4, 0xc7, 0x33, 0xc0, 0x68, 0x03, 0x04, 0x22, 0xaa, 0x9a,
            0xc3, 0xd4, 0x6c, 0x4e, 0xd2, 0x82, 0x64, 0x46, 0x07, 0x9f, 0xaa, 0x09, 0x14, 0xc2,
            0xd7, 0x05, 0xd9, 0x8b, 0x02, 0xa2, 0xb5, 0x12, 0x9c, 0xd1, 0xde, 0x16, 0x4e, 0xb9,
            0xcb, 0xd0, 0x83, 0xe8, 0xa2, 0x50, 0x3c, 0x4e,
        ];
        assert_eq!(block, expected);
    }

    #[test]
    fn masked_blocks_match_the_reference() {
        let mut rng = TestRng::new(0xc0de_1234);

        for _ in 0..8 {
            let mut key = [0_u8; 32];
            for byte in &mut key {
                *byte = rng.next_u8();
            }
            let mut nonce = [0_u8; 12];
            for byte in &mut nonce {
                *byte = rng.next_u8();
            }
            let counter = rng.next_u32();
            let expected = reference_block(&key, &nonce, counter);

            let mut entropy = TestEntropy::new(rng.next_u32().into());
            let mut cipher = MaskedChaCha20::new(&key, &nonce, counter, &mut entropy).unwrap();
            let actual = cipher.keystream_block(&mut entropy).unwrap();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn counter_advances_by_one() {
        let key = [0x42_u8; 32];
        let nonce = [0x24_u8; 12];
        let mut entropy = TestEntropy::new(0x99);
        let mut cipher = MaskedChaCha20::new(&key, &nonce, 7, &mut entropy).unwrap();

        let first = cipher.keystream_block(&mut entropy).unwrap();
        let second = cipher.keystream_block(&mut entropy).unwrap();

        assert_eq!(first, reference_block(&key, &nonce, 7));
        assert_eq!(second, reference_block(&key, &nonce, 8));
    }

    #[test]
    fn entropy_failure_is_propagated() {
        let key = [0_u8; 32];
        let nonce = [0_u8; 12];
        let mut entropy = FailingEntropy;

        assert!(matches!(
            MaskedChaCha20::new(&key, &nonce, 0, &mut entropy),
            Err(EntropyError::Unavailable)
        ));
    }

    #[test]
    fn block_entropy_failure_is_propagated() {
        let key = [0_u8; 32];
        let nonce = [0_u8; 12];
        let mut entropy = TestEntropy::new(1);
        let mut cipher = MaskedChaCha20::new(&key, &nonce, 0, &mut entropy).unwrap();
        let mut failing = FailingEntropy;

        assert!(matches!(
            cipher.keystream_block(&mut failing),
            Err(EntropyError::Unavailable)
        ));
    }

    #[test]
    fn debug_is_redacted() {
        let key = [0_u8; 32];
        let nonce = [0_u8; 12];
        let mut entropy = TestEntropy::new(1);
        let cipher = MaskedChaCha20::new(&key, &nonce, 0, &mut entropy).unwrap();
        let rendered = std::format!("{cipher:?}");

        assert_eq!(rendered, "MaskedChaCha20<redacted>");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn masked_key_shares_have_no_first_order_correlation() {
        const SAMPLES: usize = 20_000;
        const THRESHOLD: f64 = 0.03;

        let mut rng = TestRng::new(0x51a7_c0de);
        let mut entropy = TestEntropy::new(0x9876_5432);
        let nonce = [0x24_u8; 12];
        let mut key_words: [Vec<f64>; 8] = core::array::from_fn(|_| Vec::with_capacity(SAMPLES));
        let mut share0_words: [Vec<f64>; 8] = core::array::from_fn(|_| Vec::with_capacity(SAMPLES));
        let mut share1_words: [Vec<f64>; 8] = core::array::from_fn(|_| Vec::with_capacity(SAMPLES));

        for _ in 0..SAMPLES {
            let mut key = [0_u8; 32];
            for byte in &mut key {
                *byte = rng.next_u8();
            }
            let cipher = MaskedChaCha20::new(&key, &nonce, 0, &mut entropy).unwrap();

            for (index, chunk) in key.as_chunks::<4>().0.iter().enumerate() {
                key_words[index].push(f64::from(word_from_bytes(*chunk)));
                let (share0, share1) = cipher.state[4 + index].shares();
                share0_words[index].push(f64::from(*share0));
                share1_words[index].push(f64::from(*share1));
            }
        }

        for index in 0..8 {
            let correlation0 = test_pearson(&key_words[index], &share0_words[index]).unwrap();
            let correlation1 = test_pearson(&key_words[index], &share1_words[index]).unwrap();

            assert!(
                correlation0.abs() < THRESHOLD,
                "key word {index} share0 correlation {correlation0} exceeded {THRESHOLD}"
            );
            assert!(
                correlation1.abs() < THRESHOLD,
                "key word {index} share1 correlation {correlation1} exceeded {THRESHOLD}"
            );
        }
    }

    #[allow(clippy::similar_names)] // Standard mathematical notation for the two axes.
    fn test_pearson(x: &[f64], y: &[f64]) -> Option<f64> {
        if x.len() != y.len() || x.len() < 2 {
            return None;
        }

        let count = f64::from(u32::try_from(x.len()).unwrap());
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_x2 = 0.0;
        let mut sum_y2 = 0.0;
        let mut cross = 0.0;
        for (x_value, y_value) in x.iter().zip(y) {
            sum_x += x_value;
            sum_y += y_value;
            sum_x2 = x_value.mul_add(*x_value, sum_x2);
            sum_y2 = y_value.mul_add(*y_value, sum_y2);
            cross = x_value.mul_add(*y_value, cross);
        }

        let numerator = count.mul_add(cross, -(sum_x * sum_y));
        let denominator_x = count.mul_add(sum_x2, -(sum_x * sum_x)).max(0.0);
        let denominator_y = count.mul_add(sum_y2, -(sum_y * sum_y)).max(0.0);
        let denominator = (denominator_x * denominator_y).sqrt();
        if denominator <= 0.0 {
            return None;
        }
        Some(numerator / denominator)
    }
}
