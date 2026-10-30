//! First-order leakage-model checks for masked compositions.
//!
//! These checks simulate masked values with a deterministic generator and
//! measure the Pearson correlation between each share and the secret. A
//! correctly masked first-order construction should show no linear correlation
//! between an individual share and the secret.
//!
//! This is model validation, not hardware evidence. It cannot replace TVLA or
//! CPA on a physical target, and it does not prove resistance to higher-order,
//! glitching, or microarchitectural attacks.

use std::fmt;

use sicas_core::{EntropyError, EntropySource, Masked};

use crate::stats::pearson;

/// Default number of simulated samples per check.
pub const DEFAULT_SAMPLES: usize = 50_000;

/// Default deterministic generator seed.
pub const DEFAULT_SEED: u64 = 0x5eed_1234_abcd_ef01;

/// Default absolute Pearson-correlation threshold.
///
/// For 50,000 independent samples the standard error of a zero correlation is
/// about 0.0045, so 0.02 is roughly 4.5 standard errors.
pub const DEFAULT_THRESHOLD: f64 = 0.02;

/// Configuration for a leakage-model check.
#[derive(Clone, Copy, Debug)]
pub struct LeakageConfig {
    /// Number of simulated samples.
    pub samples: usize,
    /// Deterministic generator seed.
    pub seed: u64,
    /// Absolute correlation threshold.
    pub threshold: f64,
}

impl Default for LeakageConfig {
    fn default() -> Self {
        Self {
            samples: DEFAULT_SAMPLES,
            seed: DEFAULT_SEED,
            threshold: DEFAULT_THRESHOLD,
        }
    }
}

/// Result of one leakage-model check.
#[derive(Clone, Debug)]
pub struct LeakageReport {
    /// Human-readable check name.
    pub check: &'static str,
    /// Number of simulated samples.
    pub samples: usize,
    /// Largest absolute correlation observed.
    pub max_abs_correlation: f64,
    /// Signal pair that produced the largest correlation.
    pub worst_signal: &'static str,
    /// Threshold used for the check.
    pub threshold: f64,
}

impl LeakageReport {
    /// Returns `true` when the largest correlation exceeds the threshold.
    #[must_use]
    pub fn leakage(&self) -> bool {
        self.max_abs_correlation > self.threshold
    }
}

impl fmt::Display for LeakageReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "leakage-model {}: max |r| = {:.6} ({}), samples = {}, threshold = {}",
            self.check, self.max_abs_correlation, self.worst_signal, self.samples, self.threshold
        )
    }
}

/// Errors produced by leakage-model checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LeakageError {
    /// Fewer than two samples were requested.
    TooFewSamples,
    /// The threshold is not a finite positive number.
    InvalidThreshold,
    /// The deterministic entropy source failed.
    Entropy,
}

impl fmt::Display for LeakageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewSamples => {
                formatter.write_str("leakage-model check requires at least two samples")
            }
            Self::InvalidThreshold => {
                formatter.write_str("leakage-model threshold must be finite and positive")
            }
            Self::Entropy => formatter.write_str("leakage-model entropy source failed"),
        }
    }
}

impl std::error::Error for LeakageError {}

impl From<EntropyError> for LeakageError {
    fn from(_error: EntropyError) -> Self {
        Self::Entropy
    }
}

/// Runs every first-order leakage-model check.
///
/// # Errors
///
/// Returns an error for an invalid configuration or an entropy failure.
pub fn check_all(config: LeakageConfig) -> Result<Vec<LeakageReport>, LeakageError> {
    validate(config)?;
    Ok(vec![
        check_masked_byte(config)?,
        check_masked_and(config)?,
        check_masked_add(config)?,
    ])
}

/// Checks a single masked byte.
///
/// # Errors
///
/// Returns an error for an invalid configuration.
pub fn check_masked_byte(config: LeakageConfig) -> Result<LeakageReport, LeakageError> {
    validate(config)?;
    let mut rng = LeakageRng::new(config.seed ^ 0x1111_1111_1111_1111);
    let mut secrets = Vec::with_capacity(config.samples);
    let mut secret_weights = Vec::with_capacity(config.samples);
    let mut share0 = Vec::with_capacity(config.samples);
    let mut share1 = Vec::with_capacity(config.samples);
    let mut share0_weights = Vec::with_capacity(config.samples);
    let mut share1_weights = Vec::with_capacity(config.samples);

    for _ in 0..config.samples {
        let secret = rng.next_u8();
        let mask = rng.next_u8();
        let masked = Masked::<u8>::mask(secret, mask);
        let (left, right) = masked.shares();

        secrets.push(f64::from(secret));
        secret_weights.push(hamming_weight_u8(secret));
        share0.push(f64::from(*left));
        share1.push(f64::from(*right));
        share0_weights.push(hamming_weight_u8(*left));
        share1_weights.push(hamming_weight_u8(*right));
    }

    let mut tracker = Tracker::new();
    tracker.observe("share0 vs secret", &secrets, &share0);
    tracker.observe("share1 vs secret", &secrets, &share1);
    tracker.observe("HW(share0) vs HW(secret)", &secret_weights, &share0_weights);
    tracker.observe("HW(share1) vs HW(secret)", &secret_weights, &share1_weights);

    Ok(tracker.finish("masked byte", config))
}

/// Checks a masked byte AND composition.
///
/// # Errors
///
/// Returns an error for an invalid configuration.
pub fn check_masked_and(config: LeakageConfig) -> Result<LeakageReport, LeakageError> {
    validate(config)?;
    let mut rng = LeakageRng::new(config.seed ^ 0x2222_2222_2222_2222);
    let mut a_values = Vec::with_capacity(config.samples);
    let mut b_values = Vec::with_capacity(config.samples);
    let mut and_values = Vec::with_capacity(config.samples);
    let mut share0 = Vec::with_capacity(config.samples);
    let mut share1 = Vec::with_capacity(config.samples);

    for _ in 0..config.samples {
        let a = rng.next_u8();
        let b = rng.next_u8();
        let masked_a = Masked::<u8>::mask(a, rng.next_u8());
        let masked_b = Masked::<u8>::mask(b, rng.next_u8());
        let result = masked_a.and(&masked_b, rng.next_u8());
        let (left, right) = result.shares();

        a_values.push(f64::from(a));
        b_values.push(f64::from(b));
        and_values.push(f64::from(a & b));
        share0.push(f64::from(*left));
        share1.push(f64::from(*right));
    }

    let mut tracker = Tracker::new();
    tracker.observe("share0 vs a", &a_values, &share0);
    tracker.observe("share1 vs a", &a_values, &share1);
    tracker.observe("share0 vs b", &b_values, &share0);
    tracker.observe("share1 vs b", &b_values, &share1);
    tracker.observe("share0 vs a&b", &and_values, &share0);
    tracker.observe("share1 vs a&b", &and_values, &share1);

    Ok(tracker.finish("masked AND", config))
}

/// Checks a masked `u32` addition composition.
///
/// # Errors
///
/// Returns an error for an invalid configuration or an entropy failure.
pub fn check_masked_add(config: LeakageConfig) -> Result<LeakageReport, LeakageError> {
    validate(config)?;
    let mut rng = LeakageRng::new(config.seed ^ 0x3333_3333_3333_3333);
    let mut entropy = LeakageEntropy::new(config.seed ^ 0x4444_4444_4444_4444);
    let mut a_values = Vec::with_capacity(config.samples);
    let mut b_values = Vec::with_capacity(config.samples);
    let mut sum_values = Vec::with_capacity(config.samples);
    let mut share0 = Vec::with_capacity(config.samples);
    let mut share1 = Vec::with_capacity(config.samples);

    for _ in 0..config.samples {
        let a = rng.next_u32();
        let b = rng.next_u32();
        let masked_a = Masked::<u32>::mask(a, rng.next_u32());
        let masked_b = Masked::<u32>::mask(b, rng.next_u32());
        let sum = masked_a.add_masked(&masked_b, &mut entropy)?;
        let (left, right) = sum.shares();

        a_values.push(f64::from(a));
        b_values.push(f64::from(b));
        sum_values.push(f64::from(a.wrapping_add(b)));
        share0.push(f64::from(*left));
        share1.push(f64::from(*right));
    }

    let mut tracker = Tracker::new();
    tracker.observe("share0 vs a", &a_values, &share0);
    tracker.observe("share1 vs a", &a_values, &share1);
    tracker.observe("share0 vs b", &b_values, &share0);
    tracker.observe("share1 vs b", &b_values, &share1);
    tracker.observe("share0 vs a+b", &sum_values, &share0);
    tracker.observe("share1 vs a+b", &sum_values, &share1);

    Ok(tracker.finish("masked add u32", config))
}

/// Runs a negative control that must detect unmasked leakage.
///
/// The control correlates the secret with itself, so a working checker must
/// report a correlation of approximately one. This validates that the check
/// can detect a leak at all.
///
/// # Errors
///
/// Returns an error for an invalid configuration.
pub fn check_unmasked_control(config: LeakageConfig) -> Result<LeakageReport, LeakageError> {
    validate(config)?;
    let mut secrets = Vec::with_capacity(config.samples);
    let mut leaked = Vec::with_capacity(config.samples);

    for index in 0..config.samples {
        let secret = u8::try_from(index).unwrap_or(0);
        secrets.push(f64::from(secret));
        leaked.push(f64::from(secret));
    }

    let mut tracker = Tracker::new();
    tracker.observe("secret vs secret", &secrets, &leaked);

    Ok(tracker.finish("unmasked control", config))
}

fn validate(config: LeakageConfig) -> Result<(), LeakageError> {
    if config.samples < 2 {
        return Err(LeakageError::TooFewSamples);
    }
    if !config.threshold.is_finite() || config.threshold <= 0.0 {
        return Err(LeakageError::InvalidThreshold);
    }
    Ok(())
}

struct Tracker {
    max_abs_correlation: f64,
    worst_signal: &'static str,
}

impl Tracker {
    const fn new() -> Self {
        Self {
            max_abs_correlation: 0.0,
            worst_signal: "none",
        }
    }

    fn observe(&mut self, signal: &'static str, secret: &[f64], leakage: &[f64]) {
        if let Some(correlation) = pearson(secret, leakage) {
            let absolute = correlation.abs();
            if absolute > self.max_abs_correlation {
                self.max_abs_correlation = absolute;
                self.worst_signal = signal;
            }
        }
    }

    const fn finish(self, check: &'static str, config: LeakageConfig) -> LeakageReport {
        LeakageReport {
            check,
            samples: config.samples,
            max_abs_correlation: self.max_abs_correlation,
            worst_signal: self.worst_signal,
            threshold: config.threshold,
        }
    }
}

fn hamming_weight_u8(value: u8) -> f64 {
    f64::from(value.count_ones())
}

struct LeakageEntropy(LeakageRng);

impl LeakageEntropy {
    const fn new(seed: u64) -> Self {
        Self(LeakageRng::new(seed))
    }
}

impl EntropySource for LeakageEntropy {
    fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), EntropyError> {
        for byte in destination {
            *byte = self.0.next_u8();
        }
        Ok(())
    }

    fn mask_u32(&mut self) -> Result<u32, EntropyError> {
        Ok(self.0.next_u32())
    }
}

struct LeakageRng(u64);

impl LeakageRng {
    const fn new(seed: u64) -> Self {
        Self(seed)
    }

    const fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn next_u32(&mut self) -> u32 {
        u32::try_from(self.next_u64() >> 32).unwrap_or(0)
    }

    fn next_u8(&mut self) -> u8 {
        u8::try_from(self.next_u64() & 0xff).unwrap_or(0)
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
    fn negative_control_detects_unmasked_leakage() {
        let report = check_unmasked_control(LeakageConfig {
            samples: 1_000,
            seed: DEFAULT_SEED,
            threshold: 0.5,
        })
        .unwrap();

        assert!(report.leakage());
        assert!((report.max_abs_correlation - 1.0).abs() < 1e-12);
        assert_eq!(report.worst_signal, "secret vs secret");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn masked_checks_stay_below_the_default_threshold() {
        let reports = check_all(LeakageConfig::default()).unwrap();

        assert_eq!(reports.len(), 3);
        for report in &reports {
            assert!(
                !report.leakage(),
                "{} exceeded the threshold: {report}",
                report.check
            );
            assert!(report.max_abs_correlation.is_finite());
        }
    }

    #[test]
    fn small_sample_check_runs() {
        let report = check_masked_byte(LeakageConfig {
            samples: 8,
            seed: 1,
            threshold: 1.0,
        })
        .unwrap();

        assert_eq!(report.samples, 8);
        assert!(report.max_abs_correlation.is_finite());
        assert!(report.max_abs_correlation <= 1.0 + 1e-12);
    }

    #[test]
    fn invalid_configuration_is_rejected() {
        assert!(matches!(
            check_masked_byte(LeakageConfig {
                samples: 1,
                seed: 1,
                threshold: 0.02,
            }),
            Err(LeakageError::TooFewSamples)
        ));
        assert!(matches!(
            check_masked_byte(LeakageConfig {
                samples: 10,
                seed: 1,
                threshold: 0.0,
            }),
            Err(LeakageError::InvalidThreshold)
        ));
        assert!(matches!(
            check_masked_byte(LeakageConfig {
                samples: 10,
                seed: 1,
                threshold: f64::NAN,
            }),
            Err(LeakageError::InvalidThreshold)
        ));
    }

    #[test]
    fn report_threshold_is_strict() {
        let report = LeakageReport {
            check: "test",
            samples: 10,
            max_abs_correlation: 0.02,
            worst_signal: "signal",
            threshold: 0.02,
        };
        assert!(!report.leakage());

        let leaking = LeakageReport {
            max_abs_correlation: 0.020_001,
            ..report
        };
        assert!(leaking.leakage());
    }

    #[test]
    fn rng_sequence_is_stable() {
        let mut rng = LeakageRng::new(DEFAULT_SEED);
        let expected = [
            0xfde2_7e9b_9489_5d85_u64,
            0x8350_d6a9_8064_f98a,
            0x292a_4176_983c_6ca4,
            0x2096_3e41_d322_64b6,
        ];
        for value in expected {
            assert_eq!(rng.next_u64(), value);
        }

        let mut rng = LeakageRng::new(DEFAULT_SEED);
        assert_eq!(rng.next_u32(), 0xfde2_7e9b);
        assert_eq!(rng.next_u8(), 138);
    }

    #[test]
    fn entropy_sequence_is_stable() {
        let mut entropy = LeakageEntropy::new(DEFAULT_SEED);
        let mut bytes = [0_u8; 4];
        entropy.fill_bytes(&mut bytes).unwrap();
        assert_eq!(bytes, [133, 138, 164, 182]);
        assert_eq!(entropy.mask_u32().unwrap(), 0x204e_b324);

        let mut entropy = LeakageEntropy::new(DEFAULT_SEED);
        assert_eq!(entropy.mask_u32().unwrap(), 0xfde2_7e9b);
    }

    #[test]
    fn hamming_weight_is_exact() {
        assert_eq!(hamming_weight_u8(0x00), 0.0);
        assert_eq!(hamming_weight_u8(0x0f), 4.0);
        assert_eq!(hamming_weight_u8(0xff), 8.0);
    }

    #[test]
    fn tracker_keeps_the_first_signal_on_ties() {
        let secret = [1.0, 2.0, 3.0];
        let inverse = [-1.0, -2.0, -3.0];
        let mut tracker = Tracker::new();
        tracker.observe("first", &secret, &secret);
        tracker.observe("second", &secret, &inverse);

        assert_eq!(tracker.max_abs_correlation, 1.0);
        assert_eq!(tracker.worst_signal, "first");
    }

    #[test]
    fn small_reports_are_deterministic() {
        let config = LeakageConfig {
            samples: 64,
            seed: DEFAULT_SEED,
            threshold: 0.999,
        };

        let byte = check_masked_byte(config).unwrap();
        assert!((byte.max_abs_correlation - 0.099_389).abs() < 1e-5);
        assert_eq!(byte.worst_signal, "HW(share1) vs HW(secret)");

        let and = check_masked_and(config).unwrap();
        assert!((and.max_abs_correlation - 0.133_392).abs() < 1e-5);
        assert_eq!(and.worst_signal, "share0 vs a");

        let add = check_masked_add(config).unwrap();
        assert!((add.max_abs_correlation - 0.390_185).abs() < 1e-5);
        assert_eq!(add.worst_signal, "share1 vs b");
    }

    #[test]
    fn two_samples_are_accepted() {
        let report = check_masked_byte(LeakageConfig {
            samples: 2,
            seed: 1,
            threshold: 1.0,
        })
        .unwrap();
        assert_eq!(report.samples, 2);
    }

    #[test]
    fn display_and_error_text_are_populated() {
        let report = LeakageReport {
            check: "test",
            samples: 10,
            max_abs_correlation: 0.01,
            worst_signal: "share0 vs secret",
            threshold: 0.02,
        };

        assert!(report.to_string().contains("leakage-model"));
        assert!(report.to_string().contains("share0"));
        assert!(LeakageError::TooFewSamples.to_string().contains("two"));
        assert!(
            LeakageError::InvalidThreshold
                .to_string()
                .contains("threshold")
        );
        assert!(LeakageError::Entropy.to_string().contains("entropy"));
    }
}
