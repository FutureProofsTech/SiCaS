//! First-order differential power analysis (DPA) against an AES S-box output bit.
//!
//! This is offline analysis code. It is not constant-time and must not be used to
//! process live secret material.

use std::fmt;

use crate::trace::TraceSet;

/// AES substitution box, used as the nonlinear target for DPA.
const AES_SBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

/// DPA result for a single-bit difference-of-means attack.
#[derive(Clone, Debug)]
pub struct DpaReport {
    /// Key guess with the largest absolute difference.
    pub best_key: u8,
    /// Bit of the AES S-box output used for the partition.
    pub best_bit: u8,
    /// Sample index of the largest difference.
    pub best_sample: usize,
    /// Largest absolute difference of means.
    pub best_difference: f64,
    /// Signed difference of means (`one - zero`) at the best sample.
    pub best_signed_difference: f64,
}

/// Errors produced by DPA.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DpaError {
    /// No traces or plaintexts were provided.
    EmptyInput,
    /// The number of traces and plaintexts differs.
    LengthMismatch,
    /// Fewer than two traces were provided.
    TooFewTraces,
}

impl fmt::Display for DpaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => formatter.write_str("DPA input is empty"),
            Self::LengthMismatch => formatter.write_str("DPA trace and plaintext counts differ"),
            Self::TooFewTraces => formatter.write_str("DPA requires at least two traces"),
        }
    }
}

impl std::error::Error for DpaError {}

/// Runs a first-order DPA difference-of-means attack over one bit of the AES
/// S-box output `SBOX[plaintext XOR key]`.
///
/// The S-box supplies the nonlinearity that makes the partition key-dependent.
/// A single-bit partition may not uniquely recover the full key from noisy
/// traces; combine several bits or samples before treating a key as recovered.
/// The first key, bit, and sample that reaches the largest absolute difference
/// is reported.
///
/// # Errors
///
/// Returns an error for empty input, mismatched counts, or fewer than two
/// traces.
pub fn dpa_bit(traces: &TraceSet, plaintexts: &[u8]) -> Result<DpaReport, DpaError> {
    if traces.is_empty() || plaintexts.is_empty() {
        return Err(DpaError::EmptyInput);
    }
    if traces.len() != plaintexts.len() {
        return Err(DpaError::LengthMismatch);
    }
    if traces.len() < 2 {
        return Err(DpaError::TooFewTraces);
    }

    let sample_count = traces.samples_per_trace();
    let mut best_key = 0_u8;
    let mut best_bit = 0_u8;
    let mut best_sample = 0_usize;
    let mut best_difference = 0.0_f64;
    let mut best_signed_difference = 0.0_f64;
    let mut zero_sum = vec![0.0_f64; sample_count];
    let mut one_sum = vec![0.0_f64; sample_count];

    for key in u8::MIN..=u8::MAX {
        for bit in 0..8_u8 {
            zero_sum.fill(0.0);
            one_sum.fill(0.0);
            let mut zero_count = 0_usize;
            let mut one_count = 0_usize;

            for (trace, plaintext) in traces.traces().iter().zip(plaintexts) {
                #[allow(clippy::indexing_slicing)]
                // A u8 index is always inside the 256-byte table.
                let sbox_value = AES_SBOX[usize::from(plaintext ^ key)];
                let selected = (sbox_value >> bit) & 1 == 1;
                let sums = if selected {
                    &mut one_sum
                } else {
                    &mut zero_sum
                };
                if selected {
                    one_count = one_count.saturating_add(1);
                } else {
                    zero_count = zero_count.saturating_add(1);
                }

                for (sample, value) in trace.iter().enumerate() {
                    if let Some(sum) = sums.get_mut(sample) {
                        *sum += f64::from(*value);
                    }
                }
            }

            if zero_count == 0 {
                continue;
            }
            if one_count == 0 {
                continue;
            }

            let zero_mean = 1.0 / f64::from(u32::try_from(zero_count).unwrap_or(u32::MAX));
            let one_mean = 1.0 / f64::from(u32::try_from(one_count).unwrap_or(u32::MAX));

            for sample in 0..sample_count {
                let zero = zero_sum.get(sample).copied().unwrap_or(0.0) * zero_mean;
                let one = one_sum.get(sample).copied().unwrap_or(0.0) * one_mean;
                let signed = one - zero;
                let difference = signed.abs();
                if difference > best_difference {
                    best_difference = difference;
                    best_signed_difference = signed;
                    best_key = key;
                    best_bit = bit;
                    best_sample = sample;
                }
            }
        }
    }

    Ok(DpaReport {
        best_key,
        best_bit,
        best_sample,
        best_difference,
        best_signed_difference,
    })
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
        clippy::as_conversions,
        clippy::float_cmp
    )]

    use super::*;

    #[test]
    fn sbox_matches_known_aes_values() {
        assert_eq!(AES_SBOX[0x00], 0x63);
        assert_eq!(AES_SBOX[0x01], 0x7c);
        assert_eq!(AES_SBOX[0x53], 0xed);
        assert_eq!(AES_SBOX[0xff], 0x16);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native full partition sweep; Miri covers the error paths.
    fn recovers_a_single_bit_sbox_leak() {
        let secret = 0x42_u8;
        let plaintexts: Vec<u8> = (0_u8..=255).collect();
        let traces: Vec<Vec<i16>> = plaintexts
            .iter()
            .map(|plaintext| {
                let sbox_value = AES_SBOX[usize::from(plaintext ^ secret)];
                let leak = if sbox_value & 1 == 1 { 100 } else { 0 };
                vec![leak, 0]
            })
            .collect();
        let trace_set = TraceSet::from_traces(traces).unwrap();
        let report = dpa_bit(&trace_set, &plaintexts).unwrap();

        assert_eq!(report.best_key, secret);
        assert_eq!(report.best_bit, 0);
        assert_eq!(report.best_sample, 0);
        assert_eq!(report.best_difference, 100.0);
        assert_eq!(report.best_signed_difference, 100.0);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native full partition sweep; Miri covers the error paths.
    fn recovers_a_higher_bit_sbox_leak_with_negative_direction() {
        let secret = 0x37_u8;
        let plaintexts: Vec<u8> = (0_u8..=255).collect();
        let traces: Vec<Vec<i16>> = plaintexts
            .iter()
            .map(|plaintext| {
                let sbox_value = AES_SBOX[usize::from(plaintext ^ secret)];
                let leak = if (sbox_value >> 3) & 1 == 1 { 0 } else { 100 };
                vec![0, leak]
            })
            .collect();
        let trace_set = TraceSet::from_traces(traces).unwrap();
        let report = dpa_bit(&trace_set, &plaintexts).unwrap();

        assert_eq!(report.best_key, secret);
        assert_eq!(report.best_bit, 3);
        assert_eq!(report.best_sample, 1);
        assert_eq!(report.best_difference, 100.0);
        assert_eq!(report.best_signed_difference, -100.0);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Full key sweep; Miri covers the error paths.
    fn two_traces_are_accepted() {
        let traces = TraceSet::from_traces(vec![vec![1], vec![2]]).unwrap();
        let report = dpa_bit(&traces, &[0_u8, 1]).unwrap();

        assert!(report.best_difference >= 0.0);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native full partition sweep; Miri covers the error paths.
    fn constant_traces_have_zero_difference() {
        let traces = TraceSet::from_traces(vec![vec![1, 2], vec![1, 2], vec![1, 2]]).unwrap();
        let plaintexts = [0_u8, 1, 2];
        let report = dpa_bit(&traces, &plaintexts).unwrap();

        assert_eq!(report.best_difference, 0.0);
        assert_eq!(report.best_signed_difference, 0.0);
        assert_eq!(report.best_key, 0);
        assert_eq!(report.best_bit, 0);
        assert_eq!(report.best_sample, 0);
    }

    #[test]
    fn invalid_inputs_are_rejected() {
        let traces = TraceSet::from_traces(vec![vec![1], vec![2]]).unwrap();

        assert!(matches!(dpa_bit(&traces, &[]), Err(DpaError::EmptyInput)));
        assert!(matches!(
            dpa_bit(&traces, &[0_u8]),
            Err(DpaError::LengthMismatch)
        ));
        let one = TraceSet::from_traces(vec![vec![1]]).unwrap();
        assert!(matches!(
            dpa_bit(&one, &[0_u8]),
            Err(DpaError::TooFewTraces)
        ));
    }

    #[test]
    fn error_display_is_populated() {
        assert!(DpaError::EmptyInput.to_string().contains("empty"));
        assert!(DpaError::LengthMismatch.to_string().contains("counts"));
        assert!(DpaError::TooFewTraces.to_string().contains("two"));
    }
}
