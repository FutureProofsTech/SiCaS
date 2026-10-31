//! Statistical primitives used by TVLA, CPA, and timing analysis.

/// Converts a count to `f64` without an unchecked cast.
fn count_as_f64(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}

/// Returns the arithmetic mean of `values`.
#[must_use]
pub fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    let sum: f64 = values.iter().copied().sum();
    Some(sum / count_as_f64(values.len()))
}

/// Returns the sample variance of `values`.
#[must_use]
pub fn variance(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }

    let average = mean(values)?;
    let sum: f64 = values
        .iter()
        .map(|value| {
            let difference = value - average;
            difference * difference
        })
        .sum();

    Some(sum / count_as_f64(values.len().saturating_sub(1)))
}

/// Returns Welch's t-statistic for two samples.
///
/// Returns `None` when either sample has fewer than two observations. When both
/// samples have zero variance, the statistic is defined as zero.
#[must_use]
pub fn welch_t(a: &[f64], b: &[f64]) -> Option<f64> {
    if a.len() < 2 {
        return None;
    }
    if b.len() < 2 {
        return None;
    }

    let mean_a = mean(a)?;
    let mean_b = mean(b)?;
    let variance_a = variance(a)?;
    let variance_b = variance(b)?;
    let count_a = count_as_f64(a.len());
    let count_b = count_as_f64(b.len());
    let denominator = (variance_a / count_a + variance_b / count_b).sqrt();

    if denominator <= 0.0 {
        return Some(0.0);
    }

    Some((mean_a - mean_b) / denominator)
}

/// Returns Welch's t-statistic from precomputed sums.
///
/// `sum` is the sum of the observations and `sum_of_squares` is the sum of their
/// squares.
#[must_use]
pub fn welch_t_from_sums(
    count_a: usize,
    sum_a: f64,
    sum_of_squares_a: f64,
    count_b: usize,
    sum_b: f64,
    sum_of_squares_b: f64,
) -> Option<f64> {
    if count_a < 2 || count_b < 2 {
        return None;
    }

    let count_a = count_as_f64(count_a);
    let count_b = count_as_f64(count_b);
    let mean_a = sum_a / count_a;
    let mean_b = sum_b / count_b;
    let variance_a = ((sum_of_squares_a - (sum_a * sum_a / count_a)) / (count_a - 1.0)).max(0.0);
    let variance_b = ((sum_of_squares_b - (sum_b * sum_b / count_b)) / (count_b - 1.0)).max(0.0);
    let denominator = (variance_a / count_a + variance_b / count_b).sqrt();

    if denominator <= 0.0 {
        return Some(0.0);
    }

    Some((mean_a - mean_b) / denominator)
}

/// Returns Pearson's correlation coefficient for two equal-length samples.
#[allow(clippy::similar_names)] // Standard mathematical notation for the two axes.
#[must_use]
pub fn pearson(x: &[f64], y: &[f64]) -> Option<f64> {
    if x.len() != y.len() || x.len() < 2 {
        return None;
    }

    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_x2 = 0.0;
    let mut sum_y2 = 0.0;
    let mut cross_sum = 0.0;

    for (x_value, y_value) in x.iter().zip(y) {
        sum_x += *x_value;
        sum_y += *y_value;
        sum_x2 = x_value.mul_add(*x_value, sum_x2);
        sum_y2 = y_value.mul_add(*y_value, sum_y2);
        cross_sum = x_value.mul_add(*y_value, cross_sum);
    }

    pearson_from_sums(x.len(), sum_x, sum_x2, sum_y, sum_y2, cross_sum)
}

/// Returns Pearson's correlation coefficient from precomputed sums.
#[allow(clippy::similar_names)] // Standard mathematical notation for the two axes.
#[must_use]
pub fn pearson_from_sums(
    count: usize,
    sum_x: f64,
    sum_x2: f64,
    sum_y: f64,
    sum_y2: f64,
    cross_sum: f64,
) -> Option<f64> {
    if count < 2 {
        return None;
    }

    let count = count_as_f64(count);
    let numerator = count.mul_add(cross_sum, -(sum_x * sum_y));
    let denominator_x = count.mul_add(sum_x2, -(sum_x * sum_x)).max(0.0);
    let denominator_y = count.mul_add(sum_y2, -(sum_y * sum_y)).max(0.0);
    let denominator = (denominator_x * denominator_y).sqrt();

    if denominator <= 0.0 {
        return None;
    }

    Some(numerator / denominator)
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

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-12);
    }

    #[test]
    fn mean_and_variance_are_correct() {
        let values = [1.0, 2.0, 3.0, 4.0];
        assert_close(mean(&values).unwrap(), 2.5);
        assert_close(variance(&values).unwrap(), 5.0 / 3.0);
    }

    #[test]
    fn welch_t_is_zero_for_identical_samples() {
        let values = [1.0, 2.0, 3.0, 4.0];
        assert_close(welch_t(&values, &values).unwrap(), 0.0);
    }

    #[test]
    fn welch_t_detects_a_shift() {
        let a = [10.0, 10.0, 10.0, 10.0];
        let b = [0.0, 0.0, 0.0, 0.0];
        assert_eq!(welch_t(&a, &b), Some(0.0));

        let a = [10.0, 11.0, 9.0, 10.0];
        let b = [0.0, 1.0, -1.0, 0.0];
        let statistic = welch_t(&a, &b).unwrap();
        assert!(statistic > 4.5);
    }

    #[test]
    fn pearson_is_one_for_linear_relationship() {
        let x = [1.0, 2.0, 3.0, 4.0];
        let y = [2.0, 4.0, 6.0, 8.0];
        assert_close(pearson(&x, &y).unwrap(), 1.0);
    }

    #[test]
    fn pearson_is_negative_for_inverse_relationship() {
        let x = [1.0, 2.0, 3.0, 4.0];
        let y = [8.0, 6.0, 4.0, 2.0];
        assert_close(pearson(&x, &y).unwrap(), -1.0);
    }

    #[test]
    fn boundary_lengths_are_handled() {
        assert!(mean(&[]).is_none());
        assert!(variance(&[1.0]).is_none());
        assert!(variance(&[1.0, 2.0]).is_some());
        assert!(welch_t(&[1.0], &[1.0, 2.0]).is_none());
        assert!(welch_t(&[1.0, 2.0], &[1.0, 2.0]).is_some());
        assert!(pearson(&[1.0], &[1.0]).is_none());
        assert!(pearson(&[1.0, 2.0], &[1.0, 2.0]).is_some());
        assert!(pearson(&[1.0, 2.0], &[1.0]).is_none());
        assert!(pearson_from_sums(1, 0.0, 0.0, 0.0, 0.0, 0.0).is_none());
        assert!(welch_t_from_sums(1, 1.0, 1.0, 2, 0.0, 0.0).is_none());
    }

    #[test]
    fn welch_t_has_the_expected_value() {
        let a = [2.0, 4.0, 6.0, 8.0];
        let b = [0.0, 2.0, 4.0, 6.0];
        assert_close(welch_t(&a, &b).unwrap(), 1.095_445_115_010_332_1);
    }

    #[test]
    fn welch_t_from_sums_matches_the_slice_version() {
        let a = [2.0, 4.0, 6.0, 8.0];
        let b = [0.0, 2.0, 4.0, 6.0];
        let expected = welch_t(&a, &b).unwrap();
        let actual = welch_t_from_sums(
            a.len(),
            a.iter().sum(),
            a.iter().map(|value| *value * *value).sum(),
            b.len(),
            b.iter().sum(),
            b.iter().map(|value| *value * *value).sum(),
        )
        .unwrap();

        assert_close(actual, expected);
    }

    #[test]
    fn pearson_has_the_expected_value() {
        let x = [1.0, 2.0, 3.0];
        let y = [1.0, 3.0, 2.0];
        assert_close(pearson(&x, &y).unwrap(), 0.5);
    }

    #[test]
    fn pearson_from_sums_matches_the_slice_version() {
        let x = [1.0, 2.0, 3.0];
        let y = [1.0, 3.0, 2.0];
        let expected = pearson(&x, &y).unwrap();
        let actual = pearson_from_sums(
            x.len(),
            x.iter().sum(),
            x.iter().map(|value| *value * *value).sum(),
            y.iter().sum(),
            y.iter().map(|value| *value * *value).sum(),
            x.iter().zip(y.iter()).map(|(a, b)| *a * *b).sum(),
        )
        .unwrap();

        assert_close(actual, expected);
    }
}
