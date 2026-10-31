//! First-order Boolean masked ChaCha20 reference integration.
//!
//! This is a validation target, not an audited production cipher. The keystream
//! returned by [`MaskedChaCha20::keystream_block`] is a declassification
//! boundary.

use core::fmt;

use sicas_core::{EntropyError, EntropySource, Masked};

const CONSTANTS: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];

/// Number of ChaCha20 blocks processed together by
/// [`MaskedChaCha20::keystream_blocks`].
pub const BATCH_LANES: usize = 64;

type Plane = Masked<u64>;
type Planes = [Plane; 32];
type BitslicedState = [Planes; 16];

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

    /// Produces 64 consecutive ChaCha20 keystream blocks with a bitsliced masked
    /// state.
    ///
    /// This amortizes each mask over 64 independent blocks. The logical output
    /// is identical to calling [`Self::keystream_block`] 64 times. The returned
    /// bytes are declassified keystream; callers must justify this boundary.
    ///
    /// # Errors
    ///
    /// Returns [`EntropyError::Unavailable`] when the entropy source fails.
    pub fn keystream_blocks(
        &mut self,
        entropy: &mut impl EntropySource,
    ) -> Result<[[u8; 64]; BATCH_LANES], EntropyError> {
        let mut working = self.bitsliced_state(entropy)?;
        let initial = refresh_planes(&working, entropy)?;

        for _ in 0..10 {
            quarter_round_bitsliced(&mut working, 0, 4, 8, 12, entropy)?;
            quarter_round_bitsliced(&mut working, 1, 5, 9, 13, entropy)?;
            quarter_round_bitsliced(&mut working, 2, 6, 10, 14, entropy)?;
            quarter_round_bitsliced(&mut working, 3, 7, 11, 15, entropy)?;
            quarter_round_bitsliced(&mut working, 0, 5, 10, 15, entropy)?;
            quarter_round_bitsliced(&mut working, 1, 6, 11, 12, entropy)?;
            quarter_round_bitsliced(&mut working, 2, 7, 8, 13, entropy)?;
            quarter_round_bitsliced(&mut working, 3, 4, 9, 14, entropy)?;
        }

        for (word, initial_word) in working.iter_mut().zip(&initial) {
            *word = add_planes(word, initial_word, entropy)?;
        }

        let output = transpose_output(&working);
        let state_mask = entropy.mask_u32()?;
        let next_counter = self
            .counter
            .wrapping_add(u32::try_from(BATCH_LANES).unwrap_or(u32::MAX));
        self.counter = next_counter;
        if let Some(slot) = self.state.get_mut(12) {
            *slot = Masked::<u32>::mask(next_counter, state_mask);
        }

        Ok(output)
    }

    #[allow(clippy::indexing_slicing, clippy::needless_range_loop)] // Fixed state-word and bit indices.
    fn bitsliced_state(
        &self,
        entropy: &mut impl EntropySource,
    ) -> Result<BitslicedState, EntropyError> {
        let mut state: BitslicedState =
            core::array::from_fn(|_| core::array::from_fn(|_| Plane::from_parts(0, 0)));

        for (word, constant) in CONSTANTS.iter().enumerate() {
            state[word] = public_planes(*constant);
        }

        for word in 4..12 {
            state[word] = masked_word_planes(&self.state[word], entropy)?;
        }

        state[12] = counter_planes(self.counter);

        for word in 13..16 {
            let value = *self.state[word].unmask().expose_secret();
            state[word] = public_planes(value);
        }

        Ok(state)
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

#[allow(clippy::indexing_slicing)] // Fixed 32-bit plane indices.
fn public_planes(value: u32) -> Planes {
    core::array::from_fn(|bit| {
        let shift = u32::try_from(bit).unwrap_or(0);
        let set = (value >> shift) & 1 == 1;
        Plane::from_parts(if set { u64::MAX } else { 0 }, 0)
    })
}

#[allow(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::integer_division
)] // Fixed 32-bit plane and 64-lane indices.
fn counter_planes(counter: u32) -> Planes {
    core::array::from_fn(|bit| {
        let shift = u32::try_from(bit).unwrap_or(0);
        let mut plane = 0_u64;
        for lane in 0..BATCH_LANES {
            let value = counter.wrapping_add(u32::try_from(lane).unwrap_or(0));
            if (value >> shift) & 1 == 1 {
                plane |= 1_u64 << lane;
            }
        }
        Plane::from_parts(plane, 0)
    })
}

#[allow(
    clippy::indexing_slicing,
    clippy::needless_range_loop,
    clippy::similar_names
)] // Fixed 32-bit plane indices.
fn masked_word_planes(
    word: &Masked<u32>,
    entropy: &mut impl EntropySource,
) -> Result<Planes, EntropyError> {
    let (share0, share1) = word.shares();
    let mut planes: Planes = core::array::from_fn(|_| Plane::from_parts(0, 0));

    for bit in 0..32 {
        let shift = u32::try_from(bit).unwrap_or(0);
        let share0_bits = if (*share0 >> shift) & 1 == 1 {
            u64::MAX
        } else {
            0
        };
        let share1_bits = if (*share1 >> shift) & 1 == 1 {
            u64::MAX
        } else {
            0
        };
        let mask = entropy.mask_u64()?;
        planes[bit] = Plane::from_parts(share0_bits ^ mask, share1_bits ^ mask);
    }

    Ok(planes)
}

#[allow(clippy::indexing_slicing, clippy::needless_range_loop)] // Fixed state-word and bit indices.
fn refresh_planes(
    state: &BitslicedState,
    entropy: &mut impl EntropySource,
) -> Result<BitslicedState, EntropyError> {
    let mut refreshed = state.clone();
    for word in 0..16 {
        for bit in 0..32 {
            refreshed[word][bit] = refreshed[word][bit].refresh(entropy.mask_u64()?);
        }
    }
    Ok(refreshed)
}

#[allow(clippy::indexing_slicing)] // Fixed 32-bit plane indices.
fn xor_planes(a: &Planes, b: &Planes) -> Planes {
    core::array::from_fn(|bit| a[bit].xor(&b[bit]))
}

#[allow(
    clippy::indexing_slicing,
    clippy::integer_division,
    clippy::arithmetic_side_effects
)] // Fixed 32-bit plane indices.
fn rotate_left_planes(input: &Planes, amount: u32) -> Planes {
    let shift = usize::try_from(amount).unwrap_or(0) % 32;
    core::array::from_fn(|bit| input[(bit + 32 - shift) % 32].clone())
}

#[allow(clippy::indexing_slicing)] // Fixed 32-bit plane indices.
fn add_planes(
    a: &Planes,
    b: &Planes,
    entropy: &mut impl EntropySource,
) -> Result<Planes, EntropyError> {
    let mut result: Planes = core::array::from_fn(|_| Plane::from_parts(0, 0));
    let mut carry = Plane::from_parts(0, 0);

    for bit in 0..32 {
        let a_bit = &a[bit];
        let b_bit = &b[bit];
        let xor = a_bit.xor(b_bit);
        result[bit] = xor.xor(&carry);

        if bit == 31 {
            break;
        }

        let generate = a_bit.and(b_bit, entropy.mask_u64()?);
        let propagated = xor.and(&carry, entropy.mask_u64()?);
        carry = generate.xor(&propagated);
    }

    Ok(result)
}

#[allow(clippy::indexing_slicing, clippy::too_many_arguments)]
fn quarter_round_bitsliced(
    state: &mut BitslicedState,
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

    a_value = add_planes(&a_value, &b_value, entropy)?;
    d_value = rotate_left_planes(&xor_planes(&d_value, &a_value), 16);
    c_value = add_planes(&c_value, &d_value, entropy)?;
    b_value = rotate_left_planes(&xor_planes(&b_value, &c_value), 12);
    a_value = add_planes(&a_value, &b_value, entropy)?;
    d_value = rotate_left_planes(&xor_planes(&d_value, &a_value), 8);
    c_value = add_planes(&c_value, &d_value, entropy)?;
    b_value = rotate_left_planes(&xor_planes(&b_value, &c_value), 7);

    state[a] = a_value;
    state[b] = b_value;
    state[c] = c_value;
    state[d] = d_value;
    Ok(())
}

#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
fn transpose_word(planes: &Planes) -> [u32; BATCH_LANES] {
    let mut rows = [0_u64; 64];
    for bit in 0..32 {
        rows[bit] = *planes[bit].unmask().expose_secret();
    }

    let stages = [
        (32_usize, 0xffff_ffff_0000_0000_u64),
        (16, 0xffff_0000_ffff_0000),
        (8, 0xff00_ff00_ff00_ff00),
        (4, 0xf0f0_f0f0_f0f0_f0f0),
        (2, 0xcccc_cccc_cccc_cccc),
        (1, 0xaaaa_aaaa_aaaa_aaaa),
    ];

    for (width, mask) in stages {
        for row in 0..64 {
            if row & width == 0 {
                let swap = (rows[row] ^ (rows[row + width] << width)) & mask;
                rows[row] ^= swap;
                rows[row + width] ^= swap >> width;
            }
        }
    }

    core::array::from_fn(|lane| u32::try_from(rows[lane] & 0xffff_ffff).unwrap_or(0))
}

#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)] // Fixed word and lane indices.
fn transpose_output(state: &BitslicedState) -> [[u8; 64]; BATCH_LANES] {
    let mut output = [[0_u8; 64]; BATCH_LANES];

    for word in 0..16 {
        let words = transpose_word(&state[word]);
        for lane in 0..BATCH_LANES {
            output[lane][word * 4..word * 4 + 4].copy_from_slice(&words[lane].to_le_bytes());
        }
    }

    output
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

    struct FailOnMaskU32(TestRng);

    impl EntropySource for FailOnMaskU32 {
        fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), EntropyError> {
            for byte in destination {
                *byte = self.0.next_u8();
            }
            Ok(())
        }

        fn mask_u32(&mut self) -> Result<u32, EntropyError> {
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

    fn planes_from_words(words: &[u32; BATCH_LANES]) -> Planes {
        core::array::from_fn(|bit| {
            let mut plane = 0_u64;
            for (lane, word) in words.iter().enumerate() {
                if (word >> bit) & 1 == 1 {
                    plane |= 1_u64 << lane;
                }
            }
            Plane::from_parts(plane, 0)
        })
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native lane sweep; Miri covers the core gadgets.
    fn bitsliced_addition_is_lane_independent() {
        let mut rng = TestRng::new(0xadd1_7e57);
        let mut entropy = TestEntropy::new(0x1234_5678);

        for _ in 0..32 {
            let a: [u32; BATCH_LANES] = core::array::from_fn(|_| rng.next_u32());
            let b: [u32; BATCH_LANES] = core::array::from_fn(|_| rng.next_u32());
            let sum =
                add_planes(&planes_from_words(&a), &planes_from_words(&b), &mut entropy).unwrap();
            let words = transpose_word(&sum);

            for lane in 0..BATCH_LANES {
                assert_eq!(words[lane], a[lane].wrapping_add(b[lane]));
            }
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native statistical sweep; Miri covers the core gadgets.
    fn bitsliced_key_planes_are_balanced() {
        let mut entropy = TestEntropy::new(0xba1a_4ced);
        let samples = 2_048_usize;

        for bit in 0..32 {
            let mut ones = 0_usize;
            for _ in 0..samples {
                let word = Masked::<u32>::mask(u32::MAX, entropy.mask_u32().unwrap());
                let planes = masked_word_planes(&word, &mut entropy).unwrap();
                let (share0, _) = planes[bit].shares();
                ones += usize::from((*share0 & 1) == 1);
            }
            let deviation = ones.abs_diff(samples / 2);
            assert!(
                deviation < samples / 20,
                "bitplane {bit} has a biased share: {ones} ones"
            );
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native batch sweep; Miri covers the core gadgets.
    fn bitsliced_blocks_match_the_reference() {
        let mut rng = TestRng::new(0xb175_c0de);

        for _ in 0..3 {
            let mut key = [0_u8; 32];
            for byte in &mut key {
                *byte = rng.next_u8();
            }
            let mut nonce = [0_u8; 12];
            for byte in &mut nonce {
                *byte = rng.next_u8();
            }
            let counter = rng.next_u32();
            let mut entropy = TestEntropy::new(rng.next_u32().into());
            let mut cipher = MaskedChaCha20::new(&key, &nonce, counter, &mut entropy).unwrap();
            let blocks = cipher.keystream_blocks(&mut entropy).unwrap();

            for (lane, block) in blocks.iter().enumerate() {
                let expected = reference_block(
                    &key,
                    &nonce,
                    counter.wrapping_add(u32::try_from(lane).unwrap()),
                );
                assert_eq!(*block, expected, "lane {lane} mismatch");
            }
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native batch sweep; Miri covers the core gadgets.
    fn bitsliced_batch_handles_counter_wrap() {
        let key = [0x42_u8; 32];
        let nonce = [0x24_u8; 12];
        let counter = u32::MAX - 2;
        let mut entropy = TestEntropy::new(0x99);
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
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native batch sweep; Miri covers the core gadgets.
    fn scalar_then_batch_is_consistent() {
        let key = [0x42_u8; 32];
        let nonce = [0x24_u8; 12];
        let mut entropy = TestEntropy::new(0x99);
        let mut cipher = MaskedChaCha20::new(&key, &nonce, 7, &mut entropy).unwrap();

        assert_eq!(
            cipher.keystream_block(&mut entropy).unwrap(),
            reference_block(&key, &nonce, 7)
        );
        let blocks = cipher.keystream_blocks(&mut entropy).unwrap();
        assert_eq!(blocks[0], reference_block(&key, &nonce, 8));
        assert_eq!(
            blocks[BATCH_LANES - 1],
            reference_block(
                &key,
                &nonce,
                8_u32.wrapping_add(u32::try_from(BATCH_LANES - 1).unwrap())
            )
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native batch sweep; Miri covers the core gadgets.
    fn bitsliced_counter_advances_by_a_batch() {
        let key = [0x42_u8; 32];
        let nonce = [0x24_u8; 12];
        let mut entropy = TestEntropy::new(0x99);
        let mut cipher = MaskedChaCha20::new(&key, &nonce, 7, &mut entropy).unwrap();

        let blocks = cipher.keystream_blocks(&mut entropy).unwrap();
        assert_eq!(blocks[0], reference_block(&key, &nonce, 7));
        assert_eq!(
            blocks[BATCH_LANES - 1],
            reference_block(
                &key,
                &nonce,
                7_u32.wrapping_add(u32::try_from(BATCH_LANES - 1).unwrap())
            )
        );

        let next = cipher.keystream_block(&mut entropy).unwrap();
        assert_eq!(
            next,
            reference_block(
                &key,
                &nonce,
                7_u32.wrapping_add(u32::try_from(BATCH_LANES).unwrap())
            )
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native batch failure path; Miri covers core gadgets.
    fn batch_counter_mask_failure_does_not_advance_state() {
        let key = [0x42_u8; 32];
        let nonce = [0x24_u8; 12];
        let mut entropy = TestEntropy::new(1);
        let mut cipher = MaskedChaCha20::new(&key, &nonce, 0, &mut entropy).unwrap();
        let mut failing = FailOnMaskU32(TestRng::new(2));

        assert!(matches!(
            cipher.keystream_blocks(&mut failing),
            Err(EntropyError::Unavailable)
        ));

        let next = cipher.keystream_block(&mut entropy).unwrap();
        assert_eq!(next, reference_block(&key, &nonce, 0));
    }

    #[test]
    fn bitsliced_entropy_failure_is_propagated() {
        let key = [0_u8; 32];
        let nonce = [0_u8; 12];
        let mut entropy = TestEntropy::new(1);
        let mut cipher = MaskedChaCha20::new(&key, &nonce, 0, &mut entropy).unwrap();
        let mut failing = FailingEntropy;

        assert!(matches!(
            cipher.keystream_blocks(&mut failing),
            Err(EntropyError::Unavailable)
        ));

        // A failed batch must not advance the counter or desynchronize the
        // scalar and bitsliced paths.
        let next = cipher.keystream_block(&mut entropy).unwrap();
        assert_eq!(next, reference_block(&key, &nonce, 0));
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
