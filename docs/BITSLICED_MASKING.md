# Bitsliced Masking Argument

## Representation

`MaskedChaCha20::keystream_blocks` processes 64 ChaCha20 blocks together. Each 32-bit
state word is represented as 32 bitplanes. A bitplane is a `Masked<u64>` whose two shares
each hold one bit for all 64 lanes.

The key words are converted from the scalar masked state into bitplanes with fresh
`u64` masks. Constants, nonce, and counters are public and use an unmasked share. The
initial state used by the final addition is independently refreshed before the rounds.

## Operations

- XOR and rotation are linear on shares; they preserve the two-share representation.
- Addition uses a masked ripple-carry full adder:
  - `sum = a ^ b ^ carry`;
  - `carry = (a & b) ^ (carry & (a ^ b))`.
- Every masked AND uses a fresh `u64` mask. The full adder uses two masked AND gates per
  bit; the carry chain is sequential but lane-independent.
- The output is declassified only at the keystream boundary.

## Security argument

1. `Masked<u64>` and its AND/XOR/refresh operations are covered by the core Kani
   harnesses for all integer widths.
2. Each key bitplane is masked with a fresh, independent `u64` mask.
3. The initial-state bitplanes are refreshed independently, so the final addition does
   not consume correlated shares.
4. The bitsliced addition is lane-independent; the distinct-lane test compares all 64
   lanes against `wrapping_add` over 32 randomized inputs.
5. Key bitplanes are statistically balanced across 2,048 fresh masks per bit.
6. The batch output matches the independent unmasked ChaCha20 reference for randomized
   keys, nonces, and counters, including counter wraparound.
6. Entropy failure is propagated, not silently replaced.
7. Entropy failure is fail-closed and leaves the counter and scalar state
   synchronized; this is covered by early-failure, final counter-mask-failure, and
   scalar/batch interoperability tests.
8. Counter wraparound is covered by a dedicated batch test.
9. The reference mutation run has zero missed viable mutants.

## What is proven locally

- Core masked-gadget correctness: Kani.
- Bitsliced addition: differential lane test.
- Full batch composition: RFC 8439 and randomized reference tests.
- Constant-time assembly matrix: 69 configurations.
- First-order leakage model: deterministic checks with a negative control.

## Limits

- This is a first-order Boolean-masking argument, not a hardware TVLA/CPA result.
- It does not cover higher-order attacks, glitches, fault injection, or
  microarchitectural leakage.
- The 64-lane batch uses roughly 16 KiB of stack for the working and initial states,
  plus a 4 KiB output array. Embedded callers must verify stack capacity.
- The 5× performance target is not yet met; the 10× hard ceiling is met.
