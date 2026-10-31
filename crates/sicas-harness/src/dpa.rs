//! First-order differential power analysis (DPA) with a bit partition.

use std::fmt;

use crate::trace::TraceSet;

/// DPA result for a single-bit difference-of-means attack.
#[derive(Clone, Debug)]
pub struct DpaReport {
    /// Key guess with the largest absolute difference.
    pub best_key: u8,
    /// Bit of `plaintext XOR key` used for the partition.
    pub best_bit: u8,
    /// Sample index of the largest difference.
    pub best_sample: usize,
    /// Largest absolute difference of means.
    pub best_difference: f64,
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

/// Runs a first-order DPA difference-of-means attack over one bit of
/// `plaintext XOR key`.
///
/// A single-bit partition identifies that bit of the key, not necessarily the
/// full key; keys that share the selected bit produce the same partition.
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

    for key in u8::MIN..=u8::MAX {
        for bit in 0..8_u8 {
            let mut zero_sum = vec![0.0_f64; sample_count];
            let mut one_sum = vec![0.0_f64; sample_count];
            let mut zero_count = 0_usize;
            let mut one_count = 0_usize;

            for (trace, plaintext) in traces.traces().iter().zip(plaintexts) {
                let selected = ((plaintext ^ key) >> bit) & 1 == 1;
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

            if zero_count == 0 || one_count == 0 {
                continue;
            }

            let zero_mean = 1.0 / f64::from(u32::try_from(zero_count).unwrap_or(u32::MAX));
            let one_mean = 1.0 / f64::from(u32::try_from(one_count).unwrap_or(u32::MAX));

            for sample in 0..sample_count {
                let zero = zero_sum.get(sample).copied().unwrap_or(0.0) * zero_mean;
                let one = one_sum.get(sample).copied().unwrap_or(0.0) * one_mean;
                let difference = (one - zero).abs();
                if difference > best_difference {
                    best_difference = difference;
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
    fn finds_a_single_bit_partition() {
        let secret = 0x42_u8;
        let plaintexts: Vec<u8> = (0_u8..=255).collect();
        let traces: Vec<Vec<i16>> = plaintexts
            .iter()
            .map(|plaintext| {
                let leak = if (plaintext ^ secret) & 1 == 1 {
                    100
                } else {
                    0
                };
                vec![leak, 0]
            })
            .collect();
        let trace_set = TraceSet::from_traces(traces).unwrap();
        let report = dpa_bit(&trace_set, &plaintexts).unwrap();

        assert_eq!(report.best_bit, 0);
        assert_eq!(report.best_sample, 0);
        assert_eq!(report.best_difference, 100.0);
        assert_eq!((report.best_key ^ secret) & 1, 0);
    }

    #[test]
    fn constant_traces_have_zero_difference() {
        let traces = TraceSet::from_traces(vec![vec![1, 2], vec![1, 2], vec![1, 2]]).unwrap();
        let plaintexts = [0_u8, 1, 2];
        let report = dpa_bit(&traces, &plaintexts).unwrap();

        assert_eq!(report.best_difference, 0.0);
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
