//! Deterministic synthetic traces for harness validation.

use crate::trace::{TraceError, TraceSet};

/// Configuration for a single synthetic trace set.
#[derive(Clone, Copy, Debug)]
pub struct SyntheticConfig {
    /// Number of traces to generate.
    pub trace_count: usize,
    /// Number of samples per trace.
    pub samples_per_trace: usize,
    /// Secret byte used by the Hamming-weight leakage model.
    pub secret: u8,
    /// Maximum absolute noise added to each sample.
    pub noise_amplitude: u16,
    /// Leakage amplitude applied at sample zero.
    pub leakage_amplitude: i16,
    /// Deterministic generator seed.
    pub seed: u64,
}

/// A synthetic trace set and its plaintexts.
#[derive(Clone, Debug)]
pub struct SyntheticData {
    /// Generated traces.
    pub traces: TraceSet,
    /// Plaintext corresponding to each trace.
    pub plaintexts: Vec<u8>,
}

/// Configuration for fixed-vs-random synthetic groups.
#[derive(Clone, Copy, Debug)]
pub struct TvlaSyntheticConfig {
    /// Number of fixed-input traces.
    pub fixed_count: usize,
    /// Number of random-input traces.
    pub random_count: usize,
    /// Number of samples per trace.
    pub samples_per_trace: usize,
    /// Secret byte used by the Hamming-weight leakage model.
    pub secret: u8,
    /// Plaintext used for the fixed group.
    pub fixed_plaintext: u8,
    /// Maximum absolute noise added to each sample.
    pub noise_amplitude: u16,
    /// Leakage amplitude applied at sample zero.
    pub leakage_amplitude: i16,
    /// Deterministic generator seed.
    pub seed: u64,
}

/// Fixed and random synthetic groups for TVLA.
#[derive(Clone, Debug)]
pub struct TvlaSyntheticData {
    /// Fixed-input traces.
    pub fixed: TraceSet,
    /// Random-input traces.
    pub random: TraceSet,
    /// Plaintexts for the fixed group.
    pub fixed_plaintexts: Vec<u8>,
    /// Plaintexts for the random group.
    pub random_plaintexts: Vec<u8>,
}

/// Generates deterministic synthetic traces with a leak at sample zero.
///
/// # Errors
///
/// Returns an error when the dimensions are invalid or exceed trace limits.
pub fn generate(config: SyntheticConfig) -> Result<SyntheticData, TraceError> {
    let mut rng = TestRng::new(config.seed);
    let mut traces = Vec::new();
    traces
        .try_reserve_exact(config.trace_count)
        .map_err(|_| TraceError::TooLarge)?;
    let mut plaintexts = Vec::new();
    plaintexts
        .try_reserve_exact(config.trace_count)
        .map_err(|_| TraceError::TooLarge)?;

    for _ in 0..config.trace_count {
        let plaintext = rng.next_u8();
        let trace = generate_trace(
            &mut rng,
            plaintext,
            config.secret,
            config.samples_per_trace,
            config.noise_amplitude,
            config.leakage_amplitude,
        )?;
        traces.push(trace);
        plaintexts.push(plaintext);
    }

    Ok(SyntheticData {
        traces: TraceSet::from_traces(traces)?,
        plaintexts,
    })
}

/// Generates fixed-vs-random groups with a leak at sample zero.
///
/// # Errors
///
/// Returns an error when the dimensions are invalid or exceed trace limits.
pub fn generate_tvla(config: TvlaSyntheticConfig) -> Result<TvlaSyntheticData, TraceError> {
    let mut rng = TestRng::new(config.seed);
    let mut fixed = Vec::new();
    fixed
        .try_reserve_exact(config.fixed_count)
        .map_err(|_| TraceError::TooLarge)?;
    let mut fixed_plaintexts = Vec::new();
    fixed_plaintexts
        .try_reserve_exact(config.fixed_count)
        .map_err(|_| TraceError::TooLarge)?;

    for _ in 0..config.fixed_count {
        fixed.push(generate_trace(
            &mut rng,
            config.fixed_plaintext,
            config.secret,
            config.samples_per_trace,
            config.noise_amplitude,
            config.leakage_amplitude,
        )?);
        fixed_plaintexts.push(config.fixed_plaintext);
    }

    let mut random = Vec::new();
    random
        .try_reserve_exact(config.random_count)
        .map_err(|_| TraceError::TooLarge)?;
    let mut random_plaintexts = Vec::new();
    random_plaintexts
        .try_reserve_exact(config.random_count)
        .map_err(|_| TraceError::TooLarge)?;

    for _ in 0..config.random_count {
        let plaintext = rng.next_u8();
        random.push(generate_trace(
            &mut rng,
            plaintext,
            config.secret,
            config.samples_per_trace,
            config.noise_amplitude,
            config.leakage_amplitude,
        )?);
        random_plaintexts.push(plaintext);
    }

    Ok(TvlaSyntheticData {
        fixed: TraceSet::from_traces(fixed)?,
        random: TraceSet::from_traces(random)?,
        fixed_plaintexts,
        random_plaintexts,
    })
}

fn generate_trace(
    rng: &mut TestRng,
    plaintext: u8,
    secret: u8,
    samples_per_trace: usize,
    noise_amplitude: u16,
    leakage_amplitude: i16,
) -> Result<Vec<i16>, TraceError> {
    let mut trace = Vec::new();
    trace
        .try_reserve_exact(samples_per_trace)
        .map_err(|_| TraceError::TooLarge)?;

    let hamming_weight = i32::try_from((plaintext ^ secret).count_ones()).unwrap_or(0);
    let leakage = i32::from(leakage_amplitude).saturating_mul(hamming_weight);

    for sample in 0..samples_per_trace {
        let noise = i32::from(rng.next_noise(noise_amplitude));
        let value = if sample == 0 {
            leakage.saturating_add(noise)
        } else {
            noise
        };
        let clamped = value.clamp(i32::from(i16::MIN), i32::from(i16::MAX));
        trace.push(i16::try_from(clamped).unwrap_or(0));
    }

    Ok(trace)
}

#[derive(Clone, Debug)]
struct TestRng(u64);

impl TestRng {
    const fn new(seed: u64) -> Self {
        Self(seed ^ 0x9e37_79b9_7f4a_7c15)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        u32::try_from(self.0 >> 32).unwrap_or(0)
    }

    fn next_u8(&mut self) -> u8 {
        u8::try_from(self.next_u32() & 0xff).unwrap_or(0)
    }

    fn next_noise(&mut self, amplitude: u16) -> i16 {
        let amplitude = u32::from(amplitude);
        let span = amplitude.saturating_mul(2).saturating_add(1);
        let offset = self.next_u32().checked_rem(span).unwrap_or(0);
        let signed = i32::try_from(offset)
            .unwrap_or(0)
            .saturating_sub(i32::try_from(amplitude).unwrap_or(0));
        let clamped = signed.clamp(i32::from(i16::MIN), i32::from(i16::MAX));
        i16::try_from(clamped).unwrap_or(0)
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
        clippy::as_conversions
    )]

    use super::*;
    use crate::cpa::cpa_hamming_weight;
    use crate::tvla::{TvlaConfig, tvla};

    #[test]
    fn generated_dimensions_are_correct() {
        let data = generate(SyntheticConfig {
            trace_count: 16,
            samples_per_trace: 8,
            secret: 0x42,
            noise_amplitude: 10,
            leakage_amplitude: 100,
            seed: 1,
        })
        .unwrap();

        assert_eq!(data.traces.len(), 16);
        assert_eq!(data.traces.samples_per_trace(), 8);
        assert_eq!(data.plaintexts.len(), 16);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Native runs cover the full key sweep.
    fn cpa_recovers_the_synthetic_secret() {
        let secret = 0x42;
        let data = generate(SyntheticConfig {
            trace_count: 128,
            samples_per_trace: 4,
            secret,
            noise_amplitude: 0,
            leakage_amplitude: 100,
            seed: 7,
        })
        .unwrap();
        let report = cpa_hamming_weight(&data.traces, &data.plaintexts).unwrap();

        assert_eq!(report.best_key, secret);
        assert_eq!(report.best_sample, 0);
    }

    #[test]
    fn noise_is_deterministic() {
        let data = generate(SyntheticConfig {
            trace_count: 1,
            samples_per_trace: 4,
            secret: 0,
            noise_amplitude: 10,
            leakage_amplitude: 0,
            seed: 1,
        })
        .unwrap();

        assert_eq!(data.traces.trace(0), Some([2, 2, -6, -2].as_slice()));
    }

    #[test]
    fn leakage_is_clamped_to_i16() {
        let data = generate_tvla(TvlaSyntheticConfig {
            fixed_count: 1,
            random_count: 1,
            samples_per_trace: 1,
            secret: 0,
            fixed_plaintext: 0xff,
            noise_amplitude: 0,
            leakage_amplitude: i16::MAX,
            seed: 1,
        })
        .unwrap();

        assert_eq!(data.fixed.trace(0), Some(&[i16::MAX][..]));
    }

    #[test]
    fn tvla_detects_the_synthetic_leak() {
        let data = generate_tvla(TvlaSyntheticConfig {
            fixed_count: 128,
            random_count: 128,
            samples_per_trace: 4,
            secret: 0x42,
            fixed_plaintext: 0,
            noise_amplitude: 10,
            leakage_amplitude: 100,
            seed: 11,
        })
        .unwrap();
        let report = tvla(&data.fixed, &data.random, TvlaConfig::default()).unwrap();

        assert!(report.leakage());
        assert_eq!(report.max_sample, 0);
    }
}
