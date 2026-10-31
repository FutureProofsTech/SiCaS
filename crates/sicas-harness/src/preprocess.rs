//! Trace preprocessing: trimming, smoothing, and alignment.

use std::fmt;

use crate::trace::{TraceError, TraceSet};

/// Errors produced by trace preprocessing.
#[derive(Debug)]
pub enum PreprocessError {
    /// The trace set is empty.
    EmptyTrace,
    /// The requested sample range is invalid.
    InvalidRange,
    /// The moving-average window is zero.
    InvalidWindow,
    /// The reference trace index is out of range.
    InvalidReference,
    /// The requested shift limit is too large.
    InvalidShiftLimit,
    /// The trace set could not be rebuilt.
    Trace(TraceError),
}

impl fmt::Display for PreprocessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTrace => formatter.write_str("trace set is empty"),
            Self::InvalidRange => formatter.write_str("invalid sample range"),
            Self::InvalidWindow => formatter.write_str("moving-average window must be positive"),
            Self::InvalidReference => formatter.write_str("reference trace index is out of range"),
            Self::InvalidShiftLimit => formatter.write_str("shift limit is too large"),
            Self::Trace(error) => write!(formatter, "trace error: {error}"),
        }
    }
}

impl std::error::Error for PreprocessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Trace(error) => Some(error),
            _ => None,
        }
    }
}

impl From<TraceError> for PreprocessError {
    fn from(error: TraceError) -> Self {
        Self::Trace(error)
    }
}

/// Trims every trace to the half-open sample range `[start, end)`.
///
/// # Errors
///
/// Returns an error for an empty set, an invalid range, or a trace that cannot
/// be rebuilt.
pub fn trim(traces: &TraceSet, start: usize, end: usize) -> Result<TraceSet, PreprocessError> {
    if traces.is_empty() {
        return Err(PreprocessError::EmptyTrace);
    }
    if start >= end || end > traces.samples_per_trace() {
        return Err(PreprocessError::InvalidRange);
    }

    let mut rows = Vec::new();
    rows.try_reserve_exact(traces.len())
        .map_err(|_| PreprocessError::Trace(TraceError::TooLarge))?;

    for trace in traces.traces() {
        let slice = trace.get(start..end).ok_or(PreprocessError::InvalidRange)?;
        rows.push(slice.to_vec());
    }

    Ok(TraceSet::from_traces(rows)?)
}

/// Applies a centered moving average to every trace.
///
/// The window is measured in samples. Edge windows are clipped to the trace
/// boundary. A window of one leaves the trace unchanged.
///
/// # Errors
///
/// Returns an error for an empty set, a zero window, or a trace that cannot be
/// rebuilt.
pub fn moving_average(traces: &TraceSet, window: usize) -> Result<TraceSet, PreprocessError> {
    if traces.is_empty() {
        return Err(PreprocessError::EmptyTrace);
    }
    if window == 0 {
        return Err(PreprocessError::InvalidWindow);
    }

    let radius = window.checked_div(2).unwrap_or(0);
    let mut rows = Vec::new();
    rows.try_reserve_exact(traces.len())
        .map_err(|_| PreprocessError::Trace(TraceError::TooLarge))?;

    for trace in traces.traces() {
        let prefix_len = trace
            .len()
            .checked_add(1)
            .ok_or(PreprocessError::Trace(TraceError::TooLarge))?;
        let mut prefix = Vec::new();
        prefix
            .try_reserve_exact(prefix_len)
            .map_err(|_| PreprocessError::Trace(TraceError::TooLarge))?;
        prefix.push(0_i64);
        for sample in trace {
            let previous = prefix.last().copied().unwrap_or(0);
            prefix.push(previous.saturating_add(i64::from(*sample)));
        }

        let mut row = Vec::new();
        row.try_reserve_exact(trace.len())
            .map_err(|_| PreprocessError::Trace(TraceError::TooLarge))?;

        for index in 0..trace.len() {
            let start = index.saturating_sub(radius);
            let end = index
                .saturating_add(radius)
                .saturating_add(1)
                .min(trace.len());
            let sum = prefix
                .get(end)
                .copied()
                .unwrap_or(0)
                .saturating_sub(prefix.get(start).copied().unwrap_or(0));
            let count = i64::try_from(end.saturating_sub(start)).unwrap_or(1);
            let average = sum.checked_div(count).unwrap_or(0);
            row.push(i16::try_from(average).unwrap_or(0));
        }

        rows.push(row);
    }

    Ok(TraceSet::from_traces(rows)?)
}

/// Aligns every trace to a reference trace using normalized cross-correlation.
///
/// A positive shift moves a trace right; a negative shift moves it left.
/// Samples shifted outside the trace are zero-filled.
///
/// # Errors
///
/// Returns an error for an empty set, an invalid reference index, or a shift
/// limit that does not fit the supported range.
pub fn align(
    traces: &TraceSet,
    reference_index: usize,
    max_shift: usize,
) -> Result<TraceSet, PreprocessError> {
    if traces.is_empty() {
        return Err(PreprocessError::EmptyTrace);
    }
    let reference = traces
        .trace(reference_index)
        .ok_or(PreprocessError::InvalidReference)?;
    let limit = i32::try_from(max_shift).map_err(|_| PreprocessError::InvalidShiftLimit)?;

    if limit == 0 {
        return Ok(traces.clone());
    }

    let mut rows = Vec::new();
    rows.try_reserve_exact(traces.len())
        .map_err(|_| PreprocessError::Trace(TraceError::TooLarge))?;

    for trace in traces.traces() {
        let mut best_shift = 0;
        let mut best_score = f64::NEG_INFINITY;

        let minimum_shift = limit.checked_neg().unwrap_or(0);
        for shift in minimum_shift..=limit {
            let score = correlation(reference, trace, shift);
            if score > best_score {
                best_score = score;
                best_shift = shift;
            }
        }

        rows.push(shift_trace(trace, best_shift));
    }

    Ok(TraceSet::from_traces(rows)?)
}

fn correlation(reference: &[i16], trace: &[i16], shift: i32) -> f64 {
    let mut sum = 0.0;
    let mut overlap = 0_usize;

    for (index, reference_value) in reference.iter().enumerate() {
        let source = i64::try_from(index)
            .unwrap_or(0)
            .saturating_sub(i64::from(shift));
        let Ok(source_index) = usize::try_from(source) else {
            continue;
        };
        let Some(trace_value) = trace.get(source_index) else {
            continue;
        };
        sum = f64::from(*reference_value).mul_add(f64::from(*trace_value), sum);
        overlap = overlap.saturating_add(1);
    }

    if overlap == 0 {
        f64::NEG_INFINITY
    } else {
        sum / f64::from(u32::try_from(overlap).unwrap_or(u32::MAX))
    }
}

fn shift_trace(trace: &[i16], shift: i32) -> Vec<i16> {
    let mut shifted = Vec::with_capacity(trace.len());
    for index in 0..trace.len() {
        let source = i64::try_from(index)
            .unwrap_or(0)
            .saturating_sub(i64::from(shift));
        let value = usize::try_from(source)
            .ok()
            .and_then(|source_index| trace.get(source_index).copied())
            .unwrap_or(0);
        shifted.push(value);
    }
    shifted
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

    fn trace_set() -> TraceSet {
        TraceSet::from_traces(vec![
            vec![0, 0, 10, 0, 0],
            vec![0, 0, 0, 10, 0],
            vec![0, 10, 0, 0, 0],
        ])
        .unwrap()
    }

    #[test]
    fn trim_keeps_the_requested_range() {
        let trimmed = trim(&trace_set(), 1, 4).unwrap();
        assert_eq!(trimmed.samples_per_trace(), 3);
        assert_eq!(trimmed.trace(0), Some([0, 10, 0].as_slice()));
    }

    #[test]
    fn trim_rejects_invalid_ranges() {
        assert!(matches!(
            trim(&trace_set(), 4, 4),
            Err(PreprocessError::InvalidRange)
        ));
        assert!(matches!(
            trim(&trace_set(), 0, 6),
            Err(PreprocessError::InvalidRange)
        ));
    }

    #[test]
    fn trim_accepts_the_full_length() {
        let trimmed = trim(&trace_set(), 0, 5).unwrap();
        assert_eq!(trimmed.samples_per_trace(), 5);
    }

    #[test]
    fn trace_error_source_is_preserved() {
        let error = PreprocessError::Trace(TraceError::InvalidMagic);
        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn moving_average_preserves_constant_traces() {
        let traces = TraceSet::from_traces(vec![vec![5, 5, 5, 5]]).unwrap();
        let smoothed = moving_average(&traces, 3).unwrap();
        assert_eq!(smoothed.trace(0), Some([5, 5, 5, 5].as_slice()));
    }

    #[test]
    fn moving_average_smooths_an_impulse() {
        let traces = TraceSet::from_traces(vec![vec![0, 0, 9, 0, 0]]).unwrap();
        let smoothed = moving_average(&traces, 3).unwrap();
        assert_eq!(smoothed.trace(0), Some([0, 3, 3, 3, 0].as_slice()));
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Large-window stress test; Miri coverage is provided by smaller cases.
    fn moving_average_does_not_overflow_on_large_windows() {
        let traces = TraceSet::from_traces(vec![vec![i16::MAX; 100_000]]).unwrap();
        let smoothed = moving_average(&traces, 100_000).unwrap();
        assert_eq!(smoothed.trace(0).unwrap()[50_000], i16::MAX);
    }

    #[test]
    fn moving_average_rejects_zero_window() {
        assert!(matches!(
            moving_average(&trace_set(), 0),
            Err(PreprocessError::InvalidWindow)
        ));
    }

    #[test]
    fn align_recovers_a_known_shift() {
        let aligned = align(&trace_set(), 0, 3).unwrap();
        assert_eq!(aligned.trace(0), Some([0, 0, 10, 0, 0].as_slice()));
        assert_eq!(aligned.trace(1), Some([0, 0, 10, 0, 0].as_slice()));
        assert_eq!(aligned.trace(2), Some([0, 0, 10, 0, 0].as_slice()));
    }

    #[test]
    fn align_rejects_an_invalid_reference() {
        assert!(matches!(
            align(&trace_set(), 9, 3),
            Err(PreprocessError::InvalidReference)
        ));
    }

    #[test]
    fn zero_shift_limit_returns_the_input() {
        let traces = trace_set();
        let aligned = align(&traces, 0, 0).unwrap();
        assert_eq!(aligned.trace(1), traces.trace(1));
    }

    #[test]
    fn correlation_matches_the_normalized_dot_product() {
        assert_eq!(correlation(&[1, 1], &[1, 1], 0), 1.0);
        assert_eq!(correlation(&[1, -1], &[1, -1], 0), 1.0);
        assert_eq!(correlation(&[1, -1], &[-1, 1], 0), -1.0);
        assert_eq!(correlation(&[1, 2, 3], &[1, 2, 3], 1), 4.0);
    }

    #[test]
    fn correlation_without_overlap_is_negative_infinity() {
        let score = correlation(&[1], &[1], 5);
        assert!(score.is_infinite());
        assert!(score.is_sign_negative());
    }

    #[test]
    fn shift_trace_moves_and_zero_fills() {
        assert_eq!(shift_trace(&[1, 2, 3], 0), [1, 2, 3]);
        assert_eq!(shift_trace(&[1, 2, 3], 1), [0, 1, 2]);
        assert_eq!(shift_trace(&[1, 2, 3], -1), [2, 3, 0]);
        assert_eq!(shift_trace(&[1, 2, 3], 3), [0, 0, 0]);
        assert_eq!(shift_trace(&[1, 2, 3], -3), [0, 0, 0]);
    }

    #[test]
    fn align_breaks_ties_toward_the_first_shift() {
        let traces = TraceSet::from_traces(vec![vec![1, -1, 1, -1], vec![1, -1, 1, -1]]).unwrap();
        let aligned = align(&traces, 0, 2).unwrap();

        assert_eq!(aligned.trace(1), Some([1, -1, 0, 0].as_slice()));
    }

    #[test]
    fn align_recovers_every_supported_shift() {
        let reference = pseudo_random_trace(32, 0x5eed);
        for shift in -6_i32..=6 {
            let traces =
                TraceSet::from_traces(vec![reference.clone(), shift_trace(&reference, shift)])
                    .unwrap();
            let aligned = align(&traces, 0, 6).unwrap();
            let expected = shift_trace(&shift_trace(&reference, shift), -shift);

            assert_eq!(
                aligned.trace(1),
                Some(expected.as_slice()),
                "failed to recover shift {shift}"
            );
        }
    }

    #[test]
    fn trim_matches_direct_slicing() {
        let traces = TraceSet::from_traces(vec![
            vec![1, 2, 3, 4, 5],
            vec![6, 7, 8, 9, 10],
            vec![-1, -2, -3, -4, -5],
        ])
        .unwrap();
        let trimmed = trim(&traces, 1, 4).unwrap();

        for (index, trace) in traces.traces().iter().enumerate() {
            assert_eq!(trimmed.trace(index), Some(&trace[1..4]));
        }
    }

    #[test]
    fn window_of_one_leaves_traces_unchanged() {
        let traces = trace_set();
        let smoothed = moving_average(&traces, 1).unwrap();

        for index in 0..traces.len() {
            assert_eq!(smoothed.trace(index), traces.trace(index));
        }
    }

    fn pseudo_random_trace(length: usize, seed: u64) -> Vec<i16> {
        let mut state = seed;
        (0..length)
            .map(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let value = i32::try_from((state >> 33) % 2001).unwrap() - 1000;
                i16::try_from(value).unwrap()
            })
            .collect()
    }

    #[test]
    fn errors_have_display_text() {
        assert!(PreprocessError::EmptyTrace.to_string().contains("empty"));
        assert!(PreprocessError::InvalidRange.to_string().contains("range"));
        assert!(
            PreprocessError::InvalidWindow
                .to_string()
                .contains("window")
        );
        assert!(
            PreprocessError::InvalidReference
                .to_string()
                .contains("reference")
        );
        assert!(
            PreprocessError::InvalidShiftLimit
                .to_string()
                .contains("shift")
        );
    }
}
