//! Dudect-style timing evidence tooling.

use std::fmt;
use std::hint::black_box;
use std::time::Instant;

use crate::stats::{mean, welch_t};

/// Maximum number of timing samples accepted by [`dudect`].
pub const MAX_TIMING_SAMPLES: usize = 10_000_000;

/// Timing-test configuration.
#[derive(Clone, Copy, Debug)]
pub struct TimingConfig {
    /// Number of measured samples.
    pub samples: usize,
    /// Number of warm-up calls.
    pub warmup: usize,
    /// Absolute t-statistic threshold.
    pub threshold: f64,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            samples: 100_000,
            warmup: 10_000,
            threshold: 4.5,
        }
    }
}

/// Timing-test report.
#[derive(Clone, Copy, Debug)]
pub struct TimingReport {
    /// Welch's t-statistic for the two input classes.
    pub t_statistic: f64,
    /// Mean duration of the fixed-input class in nanoseconds.
    pub fixed_mean: f64,
    /// Mean duration of the random-input class in nanoseconds.
    pub random_mean: f64,
    /// Number of fixed-input samples.
    pub fixed_samples: usize,
    /// Number of random-input samples.
    pub random_samples: usize,
    /// Threshold used for the report.
    pub threshold: f64,
}

impl TimingReport {
    /// Returns `true` when the absolute t-statistic exceeds the threshold.
    #[must_use]
    pub fn leakage(&self) -> bool {
        self.t_statistic.abs() > self.threshold
    }
}

impl fmt::Display for TimingReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "timing: t = {:.4}, fixed mean = {:.2} ns, random mean = {:.2} ns, fixed={}, random={}, threshold={}",
            self.t_statistic,
            self.fixed_mean,
            self.random_mean,
            self.fixed_samples,
            self.random_samples,
            self.threshold
        )
    }
}

/// Errors produced by timing analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimingError {
    /// Fewer than two samples were requested or observed per class.
    TooFewSamples,
    /// The requested sample count exceeds [`MAX_TIMING_SAMPLES`].
    TooLarge,
    /// The threshold is not a finite positive number.
    InvalidThreshold,
}

impl fmt::Display for TimingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewSamples => formatter.write_str("timing test requires at least two samples"),
            Self::TooLarge => formatter.write_str("timing sample count exceeds the limit"),
            Self::InvalidThreshold => {
                formatter.write_str("timing threshold must be finite and positive")
            }
        }
    }
}

impl std::error::Error for TimingError {}

/// Runs a dudect-style timing test.
///
/// `operation` is called with `true` for the fixed class and `false` for the
/// random class. It must return a value derived from the operation under test;
/// the result is passed through [`black_box`] to reduce dead-code elimination.
///
/// Timing results are environment-sensitive. Use a quiet machine, pin the
/// process, run many samples, and publish the raw measurements with the report.
///
/// # Errors
///
/// Returns an error for invalid configuration, fewer than two samples, or an
/// allocation failure caused by an oversized sample count.
pub fn dudect<F>(config: TimingConfig, mut operation: F) -> Result<TimingReport, TimingError>
where
    F: FnMut(bool) -> u64,
{
    if config.samples < 2 {
        return Err(TimingError::TooFewSamples);
    }
    if config.samples > MAX_TIMING_SAMPLES {
        return Err(TimingError::TooLarge);
    }
    if !config.threshold.is_finite() || config.threshold <= 0.0 {
        return Err(TimingError::InvalidThreshold);
    }

    let mut rng = TimingRng::new(0x5eed_1234_abcd_ef01);

    for _ in 0..config.warmup {
        let class = rng.next_bool();
        black_box(operation(black_box(class)));
    }

    let mut fixed = Vec::new();
    fixed
        .try_reserve_exact(config.samples)
        .map_err(|_| TimingError::TooLarge)?;
    let mut random = Vec::new();
    random
        .try_reserve_exact(config.samples)
        .map_err(|_| TimingError::TooLarge)?;

    let mut previous = rng.next_bool();
    for index in 0..config.samples {
        let class = if index & 1 == 0 {
            rng.next_bool()
        } else {
            !previous
        };
        previous = class;

        let start = Instant::now();
        let result = operation(black_box(class));
        let elapsed = start.elapsed().as_secs_f64() * 1_000_000_000.0;
        black_box(result);

        if class {
            fixed.push(elapsed);
        } else {
            random.push(elapsed);
        }
    }

    let t_statistic = welch_t(&fixed, &random).ok_or(TimingError::TooFewSamples)?;
    let fixed_mean = mean(&fixed).ok_or(TimingError::TooFewSamples)?;
    let random_mean = mean(&random).ok_or(TimingError::TooFewSamples)?;

    Ok(TimingReport {
        t_statistic,
        fixed_mean,
        random_mean,
        fixed_samples: fixed.len(),
        random_samples: random.len(),
        threshold: config.threshold,
    })
}

#[derive(Clone, Debug)]
struct TimingRng(u64);

impl TimingRng {
    const fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        u32::try_from(self.0 >> 32).unwrap_or(0)
    }

    fn next_bool(&mut self) -> bool {
        (self.next_u32() & 1) == 1
    }
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
    fn rejects_invalid_configuration() {
        assert!(matches!(
            dudect(
                TimingConfig {
                    samples: 1,
                    warmup: 0,
                    threshold: 4.5,
                },
                |_| 0
            ),
            Err(TimingError::TooFewSamples)
        ));

        assert!(matches!(
            dudect(
                TimingConfig {
                    samples: 10,
                    warmup: 0,
                    threshold: 0.0,
                },
                |_| 0
            ),
            Err(TimingError::InvalidThreshold)
        ));

        assert!(matches!(
            dudect(
                TimingConfig {
                    samples: MAX_TIMING_SAMPLES + 1,
                    warmup: 0,
                    threshold: 4.5,
                },
                |_| 0
            ),
            Err(TimingError::TooLarge)
        ));
    }

    #[test]
    fn small_sample_counts_are_supported() {
        let report = dudect(
            TimingConfig {
                samples: 4,
                warmup: 0,
                threshold: 4.5,
            },
            u64::from,
        )
        .unwrap();

        assert_eq!(report.fixed_samples + report.random_samples, 4);
    }

    #[test]
    fn produces_a_finite_report() {
        let report = dudect(
            TimingConfig {
                samples: 100,
                warmup: 10,
                threshold: 4.5,
            },
            u64::from,
        )
        .unwrap();

        assert!(report.t_statistic.is_finite());
        assert!(report.fixed_mean.is_finite());
        assert!(report.random_mean.is_finite());
        assert!(report.fixed_mean > 0.1);
        assert!(report.random_mean > 0.1);
        assert_eq!(report.fixed_samples + report.random_samples, 100);
    }

    #[test]
    fn leakage_threshold_is_strict() {
        let report = TimingReport {
            t_statistic: 4.5,
            fixed_mean: 1.0,
            random_mean: 1.0,
            fixed_samples: 10,
            random_samples: 10,
            threshold: 4.5,
        };
        assert!(!report.leakage());

        let leaking = TimingReport {
            t_statistic: 4.500_001,
            ..report
        };
        assert!(leaking.leakage());
    }

    #[test]
    fn display_and_errors_are_populated() {
        let report = TimingReport {
            t_statistic: 0.0,
            fixed_mean: 1.0,
            random_mean: 2.0,
            fixed_samples: 10,
            random_samples: 10,
            threshold: 4.5,
        };
        assert!(report.to_string().contains("timing"));
        assert!(TimingError::TooFewSamples.to_string().contains("two"));
        assert!(TimingError::TooLarge.to_string().contains("limit"));
        assert!(
            TimingError::InvalidThreshold
                .to_string()
                .contains("threshold")
        );
    }

    #[test]
    fn rng_sequence_is_stable() {
        let mut rng = TimingRng::new(0x5eed_1234_abcd_ef01);
        let sequence: Vec<bool> = (0..8).map(|_| rng.next_bool()).collect();

        assert_eq!(
            sequence,
            [true, true, true, true, false, false, false, true]
        );
    }
}
