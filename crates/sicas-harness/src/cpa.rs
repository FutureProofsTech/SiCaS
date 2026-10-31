//! Byte-oriented correlation power analysis.

use std::fmt;

use crate::stats::pearson_from_sums;
use crate::trace::TraceSet;

/// CPA result for a Hamming-weight leakage model.
#[derive(Clone, Debug)]
pub struct CpaReport {
    /// Key guess with the largest absolute correlation.
    pub best_key: u8,
    /// Largest absolute correlation across all samples for `best_key`.
    pub best_correlation: f64,
    /// Sample index of the best correlation.
    pub best_sample: usize,
    /// Per-key maximum absolute correlation, indexed by key value.
    pub correlations: Vec<f64>,
}

/// Errors produced by CPA.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CpaError {
    /// No traces or plaintexts were provided.
    EmptyInput,
    /// The number of traces and plaintexts differs.
    LengthMismatch,
    /// Fewer than two traces were provided.
    TooFewTraces,
}

impl fmt::Display for CpaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => formatter.write_str("CPA input is empty"),
            Self::LengthMismatch => formatter.write_str("CPA trace and plaintext counts differ"),
            Self::TooFewTraces => formatter.write_str("CPA requires at least two traces"),
        }
    }
}

impl std::error::Error for CpaError {}

/// Runs CPA with the Hamming-weight model `HW(plaintext XOR key)`.
///
/// The model is intentionally simple and is intended for harness validation and
/// first-order leakage experiments, not as a replacement for an
/// algorithm-specific leakage model.
///
/// # Errors
///
/// Returns an error for empty input, mismatched trace/plaintext counts, or
/// fewer than two traces.
pub fn cpa_hamming_weight(traces: &TraceSet, plaintexts: &[u8]) -> Result<CpaReport, CpaError> {
    cpa_with_model(traces, plaintexts, |plaintext, key| {
        u8::try_from((plaintext ^ key).count_ones()).unwrap_or(0)
    })
}

/// Runs CPA with a caller-provided byte leakage model.
///
/// `leakage_model` receives the plaintext byte and key guess and returns the
/// predicted leakage byte. This supports algorithm-specific models such as an
/// AES S-box or a Hamming-weight model.
///
/// # Errors
///
/// Returns an error for empty input, mismatched trace/plaintext counts, or
/// fewer than two traces.
pub fn cpa_with_model<F>(
    traces: &TraceSet,
    plaintexts: &[u8],
    leakage_model: F,
) -> Result<CpaReport, CpaError>
where
    F: Fn(u8, u8) -> u8,
{
    if traces.is_empty() || plaintexts.is_empty() {
        return Err(CpaError::EmptyInput);
    }
    if traces.len() != plaintexts.len() {
        return Err(CpaError::LengthMismatch);
    }
    if traces.len() < 2 {
        return Err(CpaError::TooFewTraces);
    }

    let sample_count = traces.samples_per_trace();
    let mut correlations = vec![0.0; 256];
    let mut best_key = 0_u8;
    let mut best_correlation = 0.0;
    let mut best_sample = 0;

    for key in u8::MIN..=u8::MAX {
        let model: Vec<f64> = plaintexts
            .iter()
            .map(|plaintext| f64::from(leakage_model(*plaintext, key)))
            .collect();
        let model_sum: f64 = model.iter().copied().sum();
        let model_sum_of_squares: f64 = model.iter().map(|value| value * value).sum();
        let mut max_abs_correlation = 0.0;
        let mut max_sample = 0;

        for sample in 0..sample_count {
            let mut sample_sum = 0.0;
            let mut sample_sum_of_squares = 0.0;
            let mut cross_sum = 0.0;

            for (trace, model_value) in traces.traces().iter().zip(&model) {
                let observed = f64::from(trace.get(sample).copied().unwrap_or(0));
                sample_sum += observed;
                sample_sum_of_squares = observed.mul_add(observed, sample_sum_of_squares);
                cross_sum = model_value.mul_add(observed, cross_sum);
            }

            let Some(correlation) = pearson_from_sums(
                traces.len(),
                model_sum,
                model_sum_of_squares,
                sample_sum,
                sample_sum_of_squares,
                cross_sum,
            ) else {
                continue;
            };
            let absolute = correlation.abs();

            if absolute > max_abs_correlation {
                max_abs_correlation = absolute;
                max_sample = sample;
            }
        }

        if let Some(slot) = correlations.get_mut(usize::from(key)) {
            *slot = max_abs_correlation;
        }

        if max_abs_correlation > best_correlation {
            best_correlation = max_abs_correlation;
            best_key = key;
            best_sample = max_sample;
        }
    }

    Ok(CpaReport {
        best_key,
        best_correlation,
        best_sample,
        correlations,
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
    #[cfg_attr(miri, ignore)] // Native runs cover the full key sweep.
    fn recovers_a_hamming_weight_key() {
        let secret = 0x42_u8;
        let plaintexts: Vec<u8> = (0_u8..=255).collect();
        let traces: Vec<Vec<i16>> = plaintexts
            .iter()
            .map(|plaintext| {
                let leakage = i16::try_from((*plaintext ^ secret).count_ones()).unwrap() * 100;
                vec![leakage, 0, 0]
            })
            .collect();
        let trace_set = TraceSet::from_traces(traces).unwrap();
        let report = cpa_hamming_weight(&trace_set, &plaintexts).unwrap();

        assert_eq!(report.best_key, secret);
        assert_eq!(report.best_sample, 0);
        assert!(report.best_correlation > 0.99);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native runs cover the full key sweep.
    fn custom_model_recovers_a_key() {
        let secret = 0x5a_u8;
        let plaintexts: Vec<u8> = (0_u8..=255).collect();
        let traces: Vec<Vec<i16>> = plaintexts
            .iter()
            .map(|plaintext| vec![i16::from(*plaintext ^ secret) * 100])
            .collect();
        let trace_set = TraceSet::from_traces(traces).unwrap();
        let report =
            cpa_with_model(&trace_set, &plaintexts, |plaintext, key| plaintext ^ key).unwrap();

        assert_eq!(report.best_key, secret);
        assert_eq!(report.best_sample, 0);
        assert!(report.best_correlation > 0.99);
    }

    #[test]
    fn mismatched_counts_are_rejected() {
        let traces = TraceSet::from_traces(vec![vec![1, 2], vec![3, 4]]).unwrap();
        let plaintexts = [0_u8];

        assert!(matches!(
            cpa_hamming_weight(&traces, &plaintexts),
            Err(CpaError::LengthMismatch)
        ));
    }

    #[test]
    fn empty_input_is_rejected() {
        let traces = TraceSet::from_traces(vec![vec![1], vec![2]]).unwrap();

        assert!(matches!(
            cpa_hamming_weight(&traces, &[]),
            Err(CpaError::EmptyInput)
        ));
    }

    #[test]
    fn exactly_two_traces_are_supported() {
        let traces = TraceSet::from_traces(vec![vec![0, 1], vec![2, 3]]).unwrap();
        let plaintexts = [0_u8, 1_u8];

        assert!(cpa_hamming_weight(&traces, &plaintexts).is_ok());
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native runs cover the full key sweep.
    fn duplicate_leakage_samples_keep_the_first_sample() {
        let secret = 0x42_u8;
        let plaintexts: Vec<u8> = (0_u8..=255).collect();
        let traces: Vec<Vec<i16>> = plaintexts
            .iter()
            .map(|plaintext| {
                let leakage = i16::try_from((*plaintext ^ secret).count_ones()).unwrap() * 100;
                vec![leakage, leakage]
            })
            .collect();
        let trace_set = TraceSet::from_traces(traces).unwrap();
        let report = cpa_hamming_weight(&trace_set, &plaintexts).unwrap();

        assert_eq!(report.best_key, secret);
        assert_eq!(report.best_sample, 0);
    }

    #[test]
    fn constant_traces_have_zero_correlation() {
        let traces = TraceSet::from_traces(vec![vec![0], vec![0], vec![0]]).unwrap();
        let plaintexts = [0_u8, 1, 2];
        let report = cpa_hamming_weight(&traces, &plaintexts).unwrap();

        assert_eq!(report.best_correlation, 0.0);
        assert_eq!(report.best_key, 0);
        assert_eq!(report.correlations.len(), 256);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native runs cover the full key sweep.
    fn recovers_every_tested_hamming_weight_key() {
        let plaintexts: Vec<u8> = (0_u8..=255).collect();

        for secret in [0x00_u8, 0x01, 0x0f, 0x42, 0x55, 0x7f] {
            let traces: Vec<Vec<i16>> = plaintexts
                .iter()
                .map(|plaintext| {
                    let leakage = i16::try_from((*plaintext ^ secret).count_ones()).unwrap() * 100;
                    vec![0, leakage, 0]
                })
                .collect();
            let trace_set = TraceSet::from_traces(traces).unwrap();
            let report = cpa_hamming_weight(&trace_set, &plaintexts).unwrap();

            assert_eq!(report.best_key, secret);
            assert_eq!(report.best_sample, 1);
            assert!(report.best_correlation > 0.999);
            assert_eq!(
                report.correlations[usize::from(report.best_key)],
                report.best_correlation
            );
            assert!(
                report
                    .correlations
                    .iter()
                    .all(|value| *value <= report.best_correlation)
            );
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native runs cover the full key sweep.
    fn custom_model_recovers_every_tested_key() {
        let plaintexts: Vec<u8> = (0_u8..=255).collect();

        for secret in [0x00_u8, 0x5a, 0x7f] {
            let traces: Vec<Vec<i16>> = plaintexts
                .iter()
                .map(|plaintext| vec![i16::from(*plaintext ^ secret) * 100])
                .collect();
            let trace_set = TraceSet::from_traces(traces).unwrap();
            let report =
                cpa_with_model(&trace_set, &plaintexts, |plaintext, key| plaintext ^ key).unwrap();

            assert_eq!(report.best_key, secret);
            assert_eq!(report.best_sample, 0);
            assert!(report.best_correlation > 0.999);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native runs cover the full key sweep.
    fn hamming_weight_model_has_complement_symmetry() {
        let secret = 0x42_u8;
        let complement = secret ^ 0xff;
        let plaintexts: Vec<u8> = (0_u8..=255).collect();
        let traces: Vec<Vec<i16>> = plaintexts
            .iter()
            .map(|plaintext| {
                let leakage = i16::try_from((*plaintext ^ secret).count_ones()).unwrap() * 100;
                vec![leakage]
            })
            .collect();
        let trace_set = TraceSet::from_traces(traces).unwrap();
        let report = cpa_hamming_weight(&trace_set, &plaintexts).unwrap();

        assert_eq!(
            report.correlations[usize::from(secret)],
            report.correlations[usize::from(complement)]
        );
        assert_eq!(report.best_key, secret);
    }

    #[test]
    fn error_display_is_populated() {
        assert!(CpaError::EmptyInput.to_string().contains("empty"));
        assert!(CpaError::LengthMismatch.to_string().contains("counts"));
        assert!(CpaError::TooFewTraces.to_string().contains("two"));
    }
}
