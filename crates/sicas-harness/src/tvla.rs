//! Fixed-vs-random Test Vector Leakage Assessment (TVLA).

use std::fmt;
use std::io::Read;

use crate::stats::welch_t_from_sums;
use crate::trace::{TraceReader, TraceSet};

/// Default absolute t-statistic threshold for leakage detection.
pub const DEFAULT_THRESHOLD: f64 = 4.5;

/// TVLA configuration.
#[derive(Clone, Copy, Debug)]
pub struct TvlaConfig {
    /// Absolute t-statistic threshold.
    pub threshold: f64,
}

impl Default for TvlaConfig {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_THRESHOLD,
        }
    }
}

/// TVLA report.
#[derive(Clone, Copy, Debug)]
pub struct TvlaReport {
    /// Largest absolute t-statistic across all samples.
    pub max_abs_t: f64,
    /// Sample index with the largest absolute t-statistic.
    pub max_sample: usize,
    /// Number of samples whose absolute t-statistic exceeds the threshold.
    pub exceeding_samples: usize,
    /// Total number of samples analyzed.
    pub sample_count: usize,
    /// Number of fixed-input traces.
    pub fixed_traces: usize,
    /// Number of random-input traces.
    pub random_traces: usize,
    /// Threshold used for the report.
    pub threshold: f64,
}

impl TvlaReport {
    /// Returns `true` when any sample exceeds the configured threshold.
    #[must_use]
    pub const fn leakage(&self) -> bool {
        self.exceeding_samples > 0
    }
}

impl fmt::Display for TvlaReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "TVLA: max |t| = {:.4} at sample {}; {} of {} samples exceed |t| > {}; fixed={}, random={}",
            self.max_abs_t,
            self.max_sample,
            self.exceeding_samples,
            self.sample_count,
            self.threshold,
            self.fixed_traces,
            self.random_traces
        )
    }
}

/// Errors produced by TVLA.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TvlaError {
    /// One of the groups is empty.
    EmptyGroup,
    /// One of the groups has fewer than two traces.
    TooFewTraces,
    /// The groups have different numbers of samples per trace.
    DimensionMismatch,
    /// The threshold is not a finite positive number.
    InvalidThreshold,
    /// A streaming trace read failed or the file was truncated.
    TraceRead,
    /// The streaming accumulator could not be allocated.
    TooLarge,
}

impl fmt::Display for TvlaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyGroup => formatter.write_str("TVLA group is empty"),
            Self::TooFewTraces => {
                formatter.write_str("TVLA requires at least two traces per group")
            }
            Self::DimensionMismatch => formatter.write_str("TVLA trace dimensions do not match"),
            Self::InvalidThreshold => {
                formatter.write_str("TVLA threshold must be finite and positive")
            }
            Self::TraceRead => formatter.write_str("TVLA streaming trace read failed"),
            Self::TooLarge => formatter.write_str("TVLA streaming accumulator is too large"),
        }
    }
}

impl std::error::Error for TvlaError {}

/// Runs fixed-vs-random TVLA.
///
/// # Errors
///
/// Returns an error when a group is empty, has fewer than two traces, has a
/// different number of samples per trace, or the threshold is invalid.
pub fn tvla(
    fixed: &TraceSet,
    random: &TraceSet,
    config: TvlaConfig,
) -> Result<TvlaReport, TvlaError> {
    if fixed.is_empty() || random.is_empty() {
        return Err(TvlaError::EmptyGroup);
    }
    if fixed.len() < 2 {
        return Err(TvlaError::TooFewTraces);
    }
    if random.len() < 2 {
        return Err(TvlaError::TooFewTraces);
    }
    if fixed.samples_per_trace() != random.samples_per_trace() {
        return Err(TvlaError::DimensionMismatch);
    }
    if !config.threshold.is_finite() || config.threshold <= 0.0 {
        return Err(TvlaError::InvalidThreshold);
    }

    let sample_count = fixed.samples_per_trace();
    let mut max_abs_t = 0.0;
    let mut max_sample = 0;
    let mut exceeding_samples = 0_usize;

    for sample in 0..sample_count {
        let mut fixed_sum = 0.0;
        let mut fixed_sum_of_squares = 0.0;
        for trace in fixed.traces() {
            let value = f64::from(trace.get(sample).copied().unwrap_or(0));
            fixed_sum += value;
            fixed_sum_of_squares = value.mul_add(value, fixed_sum_of_squares);
        }

        let mut random_sum = 0.0;
        let mut random_sum_of_squares = 0.0;
        for trace in random.traces() {
            let value = f64::from(trace.get(sample).copied().unwrap_or(0));
            random_sum += value;
            random_sum_of_squares = value.mul_add(value, random_sum_of_squares);
        }

        let statistic = welch_t_from_sums(
            fixed.len(),
            fixed_sum,
            fixed_sum_of_squares,
            random.len(),
            random_sum,
            random_sum_of_squares,
        )
        .ok_or(TvlaError::TooFewTraces)?;
        let absolute = statistic.abs();

        if absolute > max_abs_t {
            max_abs_t = absolute;
            max_sample = sample;
        }
        if absolute > config.threshold {
            exceeding_samples = exceeding_samples.saturating_add(1);
        }
    }

    Ok(TvlaReport {
        max_abs_t,
        max_sample,
        exceeding_samples,
        sample_count,
        fixed_traces: fixed.len(),
        random_traces: random.len(),
        threshold: config.threshold,
    })
}

/// Runs fixed-vs-random TVLA over two streaming trace readers.
///
/// The accumulator is proportional to `samples_per_trace`, not to the number of
/// traces, so a large campaign can be analyzed without loading the complete
/// trace set into memory.
///
/// # Errors
///
/// Returns an error for empty groups, fewer than two traces, mismatched
/// dimensions, an invalid threshold, a trace read failure, or an allocation
/// failure.
#[allow(clippy::indexing_slicing)] // Sample indices are bounded by `sample_count`.
pub fn tvla_streaming<F: Read, R: Read>(
    fixed: &mut TraceReader<F>,
    random: &mut TraceReader<R>,
    config: TvlaConfig,
) -> Result<TvlaReport, TvlaError> {
    if fixed.trace_count() < 2 || random.trace_count() < 2 {
        return Err(TvlaError::TooFewTraces);
    }
    if fixed.samples_per_trace() != random.samples_per_trace() {
        return Err(TvlaError::DimensionMismatch);
    }
    if !config.threshold.is_finite() || config.threshold <= 0.0 {
        return Err(TvlaError::InvalidThreshold);
    }

    let sample_count = fixed.samples_per_trace();
    let mut fixed_sum = Vec::new();
    let mut fixed_sum_of_squares = Vec::new();
    let mut random_sum = Vec::new();
    let mut random_sum_of_squares = Vec::new();
    for accumulator in [
        &mut fixed_sum,
        &mut fixed_sum_of_squares,
        &mut random_sum,
        &mut random_sum_of_squares,
    ] {
        accumulator
            .try_reserve_exact(sample_count)
            .map_err(|_| TvlaError::TooLarge)?;
        accumulator.resize(sample_count, 0.0);
    }

    let mut fixed_traces = 0_usize;
    while let Some(trace) = fixed.next_trace() {
        let trace = trace.map_err(|_| TvlaError::TraceRead)?;
        if trace.len() != sample_count {
            return Err(TvlaError::DimensionMismatch);
        }
        for (sum, value) in fixed_sum.iter_mut().zip(&trace) {
            *sum += f64::from(*value);
        }
        for (sum, value) in fixed_sum_of_squares.iter_mut().zip(&trace) {
            let value = f64::from(*value);
            *sum = value.mul_add(value, *sum);
        }
        fixed_traces = fixed_traces.saturating_add(1);
    }

    let mut random_traces = 0_usize;
    while let Some(trace) = random.next_trace() {
        let trace = trace.map_err(|_| TvlaError::TraceRead)?;
        if trace.len() != sample_count {
            return Err(TvlaError::DimensionMismatch);
        }
        for (sum, value) in random_sum.iter_mut().zip(&trace) {
            *sum += f64::from(*value);
        }
        for (sum, value) in random_sum_of_squares.iter_mut().zip(&trace) {
            let value = f64::from(*value);
            *sum = value.mul_add(value, *sum);
        }
        random_traces = random_traces.saturating_add(1);
    }

    let mut max_abs_t = 0.0;
    let mut max_sample = 0;
    let mut exceeding_samples = 0_usize;
    for sample in 0..sample_count {
        let statistic = welch_t_from_sums(
            fixed_traces,
            fixed_sum[sample],
            fixed_sum_of_squares[sample],
            random_traces,
            random_sum[sample],
            random_sum_of_squares[sample],
        )
        .ok_or(TvlaError::TooFewTraces)?;
        let absolute = statistic.abs();

        if absolute > max_abs_t {
            max_abs_t = absolute;
            max_sample = sample;
        }
        if absolute > config.threshold {
            exceeding_samples = exceeding_samples.saturating_add(1);
        }
    }

    Ok(TvlaReport {
        max_abs_t,
        max_sample,
        exceeding_samples,
        sample_count,
        fixed_traces,
        random_traces,
        threshold: config.threshold,
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
    use crate::stats::welch_t;

    #[test]
    fn identical_groups_have_no_leakage() {
        let traces =
            TraceSet::from_traces(vec![vec![1, 2, 3, 4], vec![1, 2, 3, 4], vec![1, 2, 3, 4]])
                .unwrap();
        let report = tvla(&traces, &traces, TvlaConfig::default()).unwrap();

        assert_eq!(report.max_abs_t, 0.0);
        assert!(!report.leakage());
    }

    #[test]
    fn shifted_group_is_detected() {
        let fixed = TraceSet::from_traces(vec![
            vec![0, 100, 0, 0],
            vec![1, 101, 1, 1],
            vec![2, 99, 2, 2],
            vec![3, 100, 3, 3],
        ])
        .unwrap();
        let random = TraceSet::from_traces(vec![
            vec![0, 0, 0, 0],
            vec![1, 1, 1, 1],
            vec![2, 2, 2, 2],
            vec![3, 3, 3, 3],
        ])
        .unwrap();
        let report = tvla(&fixed, &random, TvlaConfig::default()).unwrap();

        assert!(report.leakage());
        assert_eq!(report.max_sample, 1);
        assert!(report.max_abs_t > DEFAULT_THRESHOLD);
    }

    #[test]
    fn mismatched_dimensions_are_rejected() {
        let fixed = TraceSet::from_traces(vec![vec![1, 2], vec![1, 2]]).unwrap();
        let random = TraceSet::from_traces(vec![vec![1, 2, 3], vec![1, 2, 3]]).unwrap();

        assert!(matches!(
            tvla(&fixed, &random, TvlaConfig::default()),
            Err(TvlaError::DimensionMismatch)
        ));
    }

    #[test]
    fn invalid_threshold_is_rejected() {
        let traces = TraceSet::from_traces(vec![vec![1], vec![2]]).unwrap();
        let config = TvlaConfig { threshold: 0.0 };

        assert!(matches!(
            tvla(&traces, &traces, config),
            Err(TvlaError::InvalidThreshold)
        ));
    }

    #[test]
    fn exact_statistic_matches_the_reference() {
        let fixed = TraceSet::from_traces(vec![vec![1], vec![2], vec![3]]).unwrap();
        let random = TraceSet::from_traces(vec![vec![0], vec![2], vec![4]]).unwrap();
        let expected = welch_t(&[1.0, 2.0, 3.0], &[0.0, 2.0, 4.0]).unwrap().abs();
        let report = tvla(&fixed, &random, TvlaConfig::default()).unwrap();

        assert!((report.max_abs_t - expected).abs() < 1e-12);
        assert_eq!(report.max_sample, 0);
    }

    #[test]
    fn streaming_matches_in_memory() {
        let fixed = TraceSet::from_traces(vec![
            vec![0, 100, 0, 0],
            vec![1, 101, 1, 1],
            vec![2, 99, 2, 2],
            vec![3, 100, 3, 3],
        ])
        .unwrap();
        let random = TraceSet::from_traces(vec![
            vec![0, 0, 0, 0],
            vec![1, 1, 1, 1],
            vec![2, 2, 2, 2],
            vec![3, 3, 3, 3],
        ])
        .unwrap();
        let mut fixed_bytes = Vec::new();
        let mut random_bytes = Vec::new();
        fixed.write_to(&mut fixed_bytes).unwrap();
        random.write_to(&mut random_bytes).unwrap();
        let mut fixed_reader = TraceReader::new(&fixed_bytes[..]).unwrap();
        let mut random_reader = TraceReader::new(&random_bytes[..]).unwrap();

        let streaming =
            tvla_streaming(&mut fixed_reader, &mut random_reader, TvlaConfig::default()).unwrap();
        let in_memory = tvla(&fixed, &random, TvlaConfig::default()).unwrap();

        assert_eq!(streaming.max_sample, in_memory.max_sample);
        assert_eq!(streaming.exceeding_samples, in_memory.exceeding_samples);
        assert_eq!(streaming.sample_count, in_memory.sample_count);
        assert!((streaming.max_abs_t - in_memory.max_abs_t).abs() < 1e-12);
    }

    #[test]
    fn streaming_rejects_dimension_mismatch() {
        let fixed = TraceSet::from_traces(vec![vec![1, 2], vec![1, 2]]).unwrap();
        let random = TraceSet::from_traces(vec![vec![1, 2, 3], vec![1, 2, 3]]).unwrap();
        let mut fixed_bytes = Vec::new();
        let mut random_bytes = Vec::new();
        fixed.write_to(&mut fixed_bytes).unwrap();
        random.write_to(&mut random_bytes).unwrap();
        let mut fixed_reader = TraceReader::new(&fixed_bytes[..]).unwrap();
        let mut random_reader = TraceReader::new(&random_bytes[..]).unwrap();

        assert!(matches!(
            tvla_streaming(&mut fixed_reader, &mut random_reader, TvlaConfig::default()),
            Err(TvlaError::DimensionMismatch)
        ));
    }

    fn header_bytes(samples_per_trace: u32, trace_count: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&crate::trace::MAGIC);
        bytes.extend_from_slice(&crate::trace::VERSION.to_le_bytes());
        bytes.extend_from_slice(&samples_per_trace.to_le_bytes());
        bytes.extend_from_slice(&trace_count.to_le_bytes());
        bytes
    }

    #[test]
    fn streaming_rejects_too_few_traces_before_reading() {
        let fixed_bytes = header_bytes(1, 1);
        let random_bytes = header_bytes(1, 2);
        let mut fixed_reader = TraceReader::new(&fixed_bytes[..]).unwrap();
        let mut random_reader = TraceReader::new(&random_bytes[..]).unwrap();

        assert!(matches!(
            tvla_streaming(&mut fixed_reader, &mut random_reader, TvlaConfig::default()),
            Err(TvlaError::TooFewTraces)
        ));
        assert_eq!(fixed_reader.read_traces(), 0);
        assert_eq!(random_reader.read_traces(), 0);
    }

    #[test]
    fn streaming_rejects_invalid_threshold_before_reading() {
        let fixed_bytes = header_bytes(1, 2);
        let random_bytes = header_bytes(1, 2);
        let mut fixed_reader = TraceReader::new(&fixed_bytes[..]).unwrap();
        let mut random_reader = TraceReader::new(&random_bytes[..]).unwrap();

        assert!(matches!(
            tvla_streaming(
                &mut fixed_reader,
                &mut random_reader,
                TvlaConfig { threshold: 0.0 }
            ),
            Err(TvlaError::InvalidThreshold)
        ));
        assert_eq!(fixed_reader.read_traces(), 0);
        assert_eq!(random_reader.read_traces(), 0);
    }

    #[test]
    fn streaming_keeps_the_first_tie_sample() {
        let fixed = TraceSet::from_traces(vec![vec![0, 10], vec![10, 0]]).unwrap();
        let random = TraceSet::from_traces(vec![vec![0, 0], vec![0, 0]]).unwrap();
        let mut fixed_bytes = Vec::new();
        let mut random_bytes = Vec::new();
        fixed.write_to(&mut fixed_bytes).unwrap();
        random.write_to(&mut random_bytes).unwrap();
        let mut fixed_reader = TraceReader::new(&fixed_bytes[..]).unwrap();
        let mut random_reader = TraceReader::new(&random_bytes[..]).unwrap();

        let report =
            tvla_streaming(&mut fixed_reader, &mut random_reader, TvlaConfig::default()).unwrap();

        assert!(report.max_abs_t > 0.0);
        assert_eq!(report.max_sample, 0);
    }

    #[test]
    fn streaming_threshold_is_strictly_exceeded() {
        let fixed =
            TraceSet::from_traces(vec![vec![0, 100], vec![1, 101], vec![2, 99], vec![3, 100]])
                .unwrap();
        let random =
            TraceSet::from_traces(vec![vec![0, 0], vec![1, 1], vec![2, 2], vec![3, 3]]).unwrap();
        let baseline = tvla(&fixed, &random, TvlaConfig::default()).unwrap();
        let mut fixed_bytes = Vec::new();
        let mut random_bytes = Vec::new();
        fixed.write_to(&mut fixed_bytes).unwrap();
        random.write_to(&mut random_bytes).unwrap();
        let mut fixed_reader = TraceReader::new(&fixed_bytes[..]).unwrap();
        let mut random_reader = TraceReader::new(&random_bytes[..]).unwrap();

        let report = tvla_streaming(
            &mut fixed_reader,
            &mut random_reader,
            TvlaConfig {
                threshold: baseline.max_abs_t,
            },
        )
        .unwrap();

        assert_eq!(report.exceeding_samples, 0);
    }

    #[test]
    fn streaming_rejects_truncated_input() {
        let fixed = TraceSet::from_traces(vec![vec![1], vec![2]]).unwrap();
        let random = TraceSet::from_traces(vec![vec![1], vec![2]]).unwrap();
        let mut fixed_bytes = Vec::new();
        let mut random_bytes = Vec::new();
        fixed.write_to(&mut fixed_bytes).unwrap();
        random.write_to(&mut random_bytes).unwrap();
        fixed_bytes.truncate(fixed_bytes.len().saturating_sub(1));
        let mut fixed_reader = TraceReader::new(&fixed_bytes[..]).unwrap();
        let mut random_reader = TraceReader::new(&random_bytes[..]).unwrap();

        assert!(matches!(
            tvla_streaming(&mut fixed_reader, &mut random_reader, TvlaConfig::default()),
            Err(TvlaError::TraceRead)
        ));
    }

    #[test]
    fn exactly_two_traces_are_supported() {
        let traces = TraceSet::from_traces(vec![vec![1], vec![2]]).unwrap();
        assert!(tvla(&traces, &traces, TvlaConfig::default()).is_ok());
    }

    #[test]
    fn too_few_traces_are_rejected() {
        let one = TraceSet::from_traces(vec![vec![1]]).unwrap();
        let two = TraceSet::from_traces(vec![vec![1], vec![2]]).unwrap();

        assert!(matches!(
            tvla(&one, &two, TvlaConfig::default()),
            Err(TvlaError::TooFewTraces)
        ));
    }

    #[test]
    fn empty_groups_are_rejected() {
        let two = TraceSet::from_traces(vec![vec![1], vec![2]]).unwrap();
        let (empty, full) = two.split_at(0).unwrap();

        assert!(matches!(
            tvla(&empty, &full, TvlaConfig::default()),
            Err(TvlaError::EmptyGroup)
        ));
        assert!(matches!(
            tvla(&full, &empty, TvlaConfig::default()),
            Err(TvlaError::EmptyGroup)
        ));
    }

    #[test]
    fn report_display_and_leakage_are_correct() {
        let report = TvlaReport {
            max_abs_t: 4.5,
            max_sample: 2,
            exceeding_samples: 0,
            sample_count: 4,
            fixed_traces: 10,
            random_traces: 10,
            threshold: 4.5,
        };
        assert!(!report.leakage());
        assert!(report.to_string().contains("TVLA"));
        assert!(report.to_string().contains("max"));

        let leaking = TvlaReport {
            exceeding_samples: 1,
            ..report
        };
        assert!(leaking.leakage());
    }

    #[test]
    fn exact_statistic_matches_an_asymmetric_reference() {
        let fixed = TraceSet::from_traces(vec![vec![1], vec![2], vec![4]]).unwrap();
        let random = TraceSet::from_traces(vec![vec![0], vec![3], vec![5]]).unwrap();
        let expected = welch_t(&[1.0, 2.0, 4.0], &[0.0, 3.0, 5.0]).unwrap().abs();
        let report = tvla(&fixed, &random, TvlaConfig::default()).unwrap();

        assert!((report.max_abs_t - expected).abs() < 1e-12);
        assert!(report.max_abs_t > 0.0);
    }

    #[test]
    fn constant_shift_with_zero_variance_is_detected() {
        // Both groups are deterministic and differ, so the t-statistic is
        // infinite. A zero result here would be a false negative.
        let fixed = TraceSet::from_traces(vec![vec![100], vec![100]]).unwrap();
        let random = TraceSet::from_traces(vec![vec![0], vec![0]]).unwrap();
        let report = tvla(&fixed, &random, TvlaConfig::default()).unwrap();

        assert!(report.leakage());
        assert!(report.max_abs_t.is_infinite());
        assert_eq!(report.max_sample, 0);
    }

    #[test]
    fn equal_statistics_keep_the_first_sample() {
        // Both samples have the same absolute t-statistic but no zero
        // variance, so the tie-break path is exercised without masking a
        // deterministic leak.
        let fixed = TraceSet::from_traces(vec![vec![0, 10], vec![10, 0]]).unwrap();
        let random = TraceSet::from_traces(vec![vec![0, 0], vec![0, 0]]).unwrap();
        let report = tvla(&fixed, &random, TvlaConfig::default()).unwrap();

        assert!(report.max_abs_t > 0.0);
        assert_eq!(report.max_sample, 0);
    }

    #[test]
    fn threshold_is_strictly_exceeded() {
        let fixed =
            TraceSet::from_traces(vec![vec![0, 100], vec![1, 101], vec![2, 99], vec![3, 100]])
                .unwrap();
        let random =
            TraceSet::from_traces(vec![vec![0, 0], vec![1, 1], vec![2, 2], vec![3, 3]]).unwrap();
        let baseline = tvla(&fixed, &random, TvlaConfig::default()).unwrap();
        assert!(baseline.exceeding_samples > 0);

        let exact = tvla(
            &fixed,
            &random,
            TvlaConfig {
                threshold: baseline.max_abs_t,
            },
        )
        .unwrap();
        assert_eq!(exact.exceeding_samples, 0);
    }

    #[test]
    fn error_display_is_populated() {
        assert!(TvlaError::EmptyGroup.to_string().contains("empty"));
        assert!(TvlaError::TooFewTraces.to_string().contains("two"));
        assert!(
            TvlaError::DimensionMismatch
                .to_string()
                .contains("dimensions")
        );
        assert!(
            TvlaError::InvalidThreshold
                .to_string()
                .contains("threshold")
        );
    }
}
