//! Deterministic statistics fixtures for differential validation.
//!
//! The output is parsed by `tools/check_stats.py`, which recomputes each result
//! with an independent Python implementation.

#![allow(
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::indexing_slicing,
    clippy::integer_division,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::missing_const_for_fn
)]

use sicas_harness::stats::{mean, pearson, variance, welch_t};

const CASES: usize = 200;

struct Rng(u64);

impl Rng {
    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 32) as u32
    }

    fn next_f64(&mut self) -> f64 {
        f64::from(self.next_u32() % 20_001) - 10_000.0
    }
}

fn values(rng: &mut Rng, length: usize) -> Vec<f64> {
    (0..length).map(|_| rng.next_f64()).collect()
}

fn render(values: &[f64]) -> String {
    values
        .iter()
        .map(|value| format!("{value:.17e}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn main() {
    let mut rng = Rng(0x5eed_1234_abcd_ef01);

    for case in 0..CASES {
        let length_a = 2 + (rng.next_u32() as usize % 19);
        let length_b = 2 + (rng.next_u32() as usize % 19);
        let a = values(&mut rng, length_a);
        let b = values(&mut rng, length_b);

        let mean_a = mean(&a).unwrap();
        let mean_b = mean(&b).unwrap();
        let variance_a = variance(&a).unwrap();
        let variance_b = variance(&b).unwrap();
        let statistic = welch_t(&a, &b).unwrap();
        let correlation = pearson(&a, &b).unwrap_or(f64::NAN);

        println!(
            "{case}|{}|{}|{mean_a:.17e}|{mean_b:.17e}|{variance_a:.17e}|{variance_b:.17e}|{statistic:.17e}|{correlation:.17e}",
            render(&a),
            render(&b)
        );
    }
}
