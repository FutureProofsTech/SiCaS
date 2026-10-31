//! Command-line interface for the SiCaS leakage-assessment harness.

use std::env;
use std::fmt::Write as _;
use std::fs::File;
use std::hint::black_box;
use std::process::ExitCode;

use sicas_core::{Choice, EntropyError, EntropySource, Masked, ct};
use sicas_harness::cpa::cpa_hamming_weight;
use sicas_harness::dpa::dpa_bit;
use sicas_harness::leakage::{
    DEFAULT_SAMPLES, DEFAULT_SEED, DEFAULT_THRESHOLD, LeakageConfig, check_all,
    check_unmasked_control,
};
use sicas_harness::preprocess::{align, moving_average, trim};
use sicas_harness::synthetic::{SyntheticConfig, TvlaSyntheticConfig, generate, generate_tvla};
use sicas_harness::timing::{TimingConfig, TimingReport, dudect};
use sicas_harness::trace::TraceSet;
use sicas_harness::tvla::{TvlaConfig, tvla};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let Some(command) = args.get(1) else {
        print_usage();
        return Err("missing command".to_string());
    };

    let command_args = args.get(2..).unwrap_or(&[]);

    match command.as_str() {
        "tvla" => run_tvla(command_args),
        "cpa" => run_cpa(command_args),
        "dpa" => run_dpa(command_args),
        "synthetic" => run_synthetic_tvla(command_args),
        "synthetic-cpa" => run_synthetic_cpa(command_args),
        "timing" => run_timing(command_args),
        "ct" => run_ct(command_args),
        "leakage-check" => run_leakage_check(command_args),
        "trim" => run_trim(command_args),
        "filter" => run_filter(command_args),
        "align" => run_align(command_args),
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(())
        }
        other => Err(format!("unknown command: {other}")),
    }
}

fn run_tvla(args: &[String]) -> Result<(), String> {
    let fixed_path = required(args, "--fixed")?;
    let random_path = required(args, "--random")?;
    let threshold = optional_f64(args, "--threshold")?.unwrap_or(4.5);
    let fixed = read_trace_set(fixed_path)?;
    let random = read_trace_set(random_path)?;
    let report =
        tvla(&fixed, &random, TvlaConfig { threshold }).map_err(|error| error.to_string())?;

    println!("{report}");
    if report.leakage() {
        return Err("TVLA detected leakage".to_string());
    }
    Ok(())
}

fn run_cpa(args: &[String]) -> Result<(), String> {
    let traces_path = required(args, "--traces")?;
    let plaintexts_path = required(args, "--plaintexts")?;
    let traces = read_trace_set(traces_path)?;
    let plaintexts = std::fs::read(plaintexts_path)
        .map_err(|error| format!("failed to read plaintexts: {error}"))?;
    let report = cpa_hamming_weight(&traces, &plaintexts).map_err(|error| error.to_string())?;

    println!(
        "CPA: best key = 0x{:02x}, |corr| = {:.6}, sample = {}",
        report.best_key, report.best_correlation, report.best_sample
    );
    Ok(())
}

fn run_dpa(args: &[String]) -> Result<(), String> {
    let traces_path = required(args, "--traces")?;
    let plaintexts_path = required(args, "--plaintexts")?;
    let traces = read_trace_set(traces_path)?;
    let plaintexts = std::fs::read(plaintexts_path)
        .map_err(|error| format!("failed to read plaintexts: {error}"))?;
    let report = dpa_bit(&traces, &plaintexts).map_err(|error| error.to_string())?;

    println!(
        "DPA: best key = 0x{:02x}, bit = {}, |difference| = {:.3}, signed = {:.3}, sample = {}",
        report.best_key,
        report.best_bit,
        report.best_difference,
        report.best_signed_difference,
        report.best_sample
    );
    Ok(())
}

fn run_synthetic_tvla(args: &[String]) -> Result<(), String> {
    let fixed_path = required(args, "--fixed-out")?;
    let random_path = required(args, "--random-out")?;
    let config = TvlaSyntheticConfig {
        fixed_count: required_usize(args, "--fixed-count")?,
        random_count: required_usize(args, "--random-count")?,
        samples_per_trace: required_usize(args, "--samples")?,
        secret: required_u8(args, "--secret")?,
        fixed_plaintext: optional_u8(args, "--fixed-plaintext")?.unwrap_or(0),
        noise_amplitude: optional_u16(args, "--noise")?.unwrap_or(10),
        leakage_amplitude: optional_i16(args, "--leakage")?.unwrap_or(100),
        seed: optional_u64(args, "--seed")?.unwrap_or(1),
    };
    let data = generate_tvla(config).map_err(|error| error.to_string())?;
    write_trace_set(fixed_path, &data.fixed)?;
    write_trace_set(random_path, &data.random)?;

    println!(
        "wrote fixed={} random={} samples={}",
        data.fixed.len(),
        data.random.len(),
        data.fixed.samples_per_trace()
    );
    Ok(())
}

fn run_synthetic_cpa(args: &[String]) -> Result<(), String> {
    let traces_path = required(args, "--out")?;
    let plaintexts_path = required(args, "--plaintexts-out")?;
    let config = SyntheticConfig {
        trace_count: required_usize(args, "--count")?,
        samples_per_trace: required_usize(args, "--samples")?,
        secret: required_u8(args, "--secret")?,
        noise_amplitude: optional_u16(args, "--noise")?.unwrap_or(10),
        leakage_amplitude: optional_i16(args, "--leakage")?.unwrap_or(100),
        seed: optional_u64(args, "--seed")?.unwrap_or(1),
    };
    let data = generate(config).map_err(|error| error.to_string())?;
    write_trace_set(traces_path, &data.traces)?;
    std::fs::write(plaintexts_path, &data.plaintexts)
        .map_err(|error| format!("failed to write plaintexts: {error}"))?;

    println!(
        "wrote traces={} samples={} plaintexts={}",
        data.traces.len(),
        data.traces.samples_per_trace(),
        data.plaintexts.len()
    );
    Ok(())
}

fn run_timing(args: &[String]) -> Result<(), String> {
    let config = TimingConfig {
        samples: optional_usize(args, "--samples")?.unwrap_or(100_000),
        warmup: optional_usize(args, "--warmup")?.unwrap_or(10_000),
        threshold: optional_f64(args, "--threshold")?.unwrap_or(4.5),
    };
    let report = dudect(config, |class| {
        let (left, right) = if class {
            (0x1234_5678_u32, 0x1234_5678_u32)
        } else {
            (0x1234_5678_u32, 0x9abc_def0_u32)
        };
        u64::from(ct::eq_u32(black_box(left), black_box(right)).as_u8())
    })
    .map_err(|error| error.to_string())?;

    println!("{report}");
    if report.leakage() {
        return Err("timing test exceeded the leakage threshold".to_string());
    }
    Ok(())
}

fn run_leakage_check(args: &[String]) -> Result<(), String> {
    let config = LeakageConfig {
        samples: optional_usize(args, "--samples")?.unwrap_or(DEFAULT_SAMPLES),
        seed: optional_u64(args, "--seed")?.unwrap_or(DEFAULT_SEED),
        threshold: optional_f64(args, "--threshold")?.unwrap_or(DEFAULT_THRESHOLD),
    };

    let control = check_unmasked_control(config).map_err(|error| error.to_string())?;
    if !control.leakage() {
        return Err("leakage-model negative control did not detect unmasked leakage".to_string());
    }
    println!("{control}");

    let reports = check_all(config).map_err(|error| error.to_string())?;
    let mut leaked = false;
    for report in &reports {
        println!("{report}");
        if report.leakage() {
            leaked = true;
        }
    }

    if leaked {
        return Err("first-order leakage model exceeded the threshold".to_string());
    }
    Ok(())
}

struct FixedEntropy;

impl EntropySource for FixedEntropy {
    fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), EntropyError> {
        for byte in destination {
            *byte = black_box(0x5a);
        }
        Ok(())
    }
}

struct CtCheck {
    name: &'static str,
    reports: Vec<TimingReport>,
}

fn run_ct(args: &[String]) -> Result<(), String> {
    let samples = optional_usize(args, "--samples")?.unwrap_or(100_000);
    let warmup = optional_usize(args, "--warmup")?.unwrap_or(10_000);
    let trials = optional_usize(args, "--trials")?.unwrap_or(3);
    let threshold = optional_f64(args, "--threshold")?.unwrap_or(4.5);
    let report_path = optional(args, "--report");
    let label = optional(args, "--label").unwrap_or("unspecified");

    if trials == 0 {
        return Err("--trials must be at least 1".to_string());
    }

    let config = TimingConfig {
        samples,
        warmup,
        threshold,
    };

    let mut checks = Vec::new();
    checks.push(CtCheck {
        name: "ct::eq_u32",
        reports: run_trials(config, trials, |class| {
            let class_mask = u32::from(class).wrapping_neg();
            let left = 0x1234_5678_u32;
            let right = (0x1234_5678_u32 & !class_mask) | (0x9abc_def0_u32 & class_mask);
            u64::from(ct::eq_u32(black_box(left), black_box(right)).as_u8())
        })?,
    });
    checks.push(CtCheck {
        name: "ct::select_u32",
        reports: run_trials(config, trials, |class| {
            let choice = Choice::from_bool(class);
            u64::from(ct::select_u32(
                choice,
                black_box(0x1111_1111),
                black_box(0x2222_2222),
            ))
        })?,
    });
    checks.push(CtCheck {
        name: "ct::lookup_u8",
        reports: {
            let table: [u8; 256] = core::array::from_fn(|index| u8::try_from(index).unwrap_or(0));
            let table = black_box(&table);
            run_trials(config, trials, |class| {
                let index = u8::from(class).wrapping_neg();
                u64::from(ct::lookup_u8(table, black_box(index)))
            })?
        },
    });
    checks.push(CtCheck {
        name: "Masked::add_masked",
        reports: run_trials(config, trials, |class| {
            let class_mask = u32::from(class).wrapping_neg();
            let a = (0x1111_1111_u32 & !class_mask) | (0x3333_3333_u32 & class_mask);
            let b = (0x2222_2222_u32 & !class_mask) | (0x4444_4444_u32 & class_mask);
            let masked_a = Masked::<u32>::mask(black_box(a), black_box(0xaaaa_aaaa));
            let masked_b = Masked::<u32>::mask(black_box(b), black_box(0x5555_5555));
            let mut entropy = FixedEntropy;

            masked_a
                .add_masked(&masked_b, &mut entropy)
                .map_or(0, |sum| u64::from(*sum.unmask().expose_secret()))
        })?,
    });

    print_ct_report(&checks, threshold, label, samples, warmup, trials);
    if let Some(path) = report_path {
        write_ct_report(path, &checks, threshold, label, samples, warmup, trials)?;
    }

    if checks
        .iter()
        .any(|check| check.reports.iter().any(TimingReport::leakage))
    {
        return Err("constant-time timing check exceeded the threshold".to_string());
    }
    Ok(())
}

fn run_trials<F>(
    config: TimingConfig,
    trials: usize,
    mut operation: F,
) -> Result<Vec<TimingReport>, String>
where
    F: FnMut(bool) -> u64,
{
    let mut reports = Vec::new();
    reports
        .try_reserve_exact(trials)
        .map_err(|_| "failed to allocate timing reports".to_string())?;
    for _ in 0..trials {
        reports.push(dudect(config, &mut operation).map_err(|error| error.to_string())?);
    }
    Ok(reports)
}

fn print_ct_report(
    checks: &[CtCheck],
    threshold: f64,
    label: &str,
    samples: usize,
    warmup: usize,
    trials: usize,
) {
    println!(
        "constant-time timing evidence ({label}; samples={samples}, warmup={warmup}, trials={trials}, threshold |t| > {threshold})"
    );
    for check in checks {
        for (trial, report) in check.reports.iter().enumerate() {
            println!("{} trial {}: {report}", check.name, trial.saturating_add(1));
        }
    }
}

fn write_ct_report(
    path: &str,
    checks: &[CtCheck],
    threshold: f64,
    label: &str,
    samples: usize,
    warmup: usize,
    trials: usize,
) -> Result<(), String> {
    let mut report = String::new();
    report.push_str("# Constant-Time Timing Evidence\n\n");
    writeln!(report, "Label: {label}")
        .map_err(|error| format!("failed to build report: {error}"))?;
    writeln!(report, "Samples per trial: {samples}")
        .map_err(|error| format!("failed to build report: {error}"))?;
    writeln!(report, "Warmup: {warmup}")
        .map_err(|error| format!("failed to build report: {error}"))?;
    writeln!(report, "Trials: {trials}")
        .map_err(|error| format!("failed to build report: {error}"))?;
    writeln!(report, "Threshold: |t| > {threshold}\n")
        .map_err(|error| format!("failed to build report: {error}"))?;
    report.push_str("| Check | Trial | t | Fixed mean (ns) | Random mean (ns) | Result |\n");
    report.push_str("|---|---|---|---|---|---|\n");
    for check in checks {
        for (trial, result) in check.reports.iter().enumerate() {
            let verdict = if result.leakage() { "LEAK" } else { "pass" };
            writeln!(
                report,
                "| {} | {} | {:.4} | {:.2} | {:.2} | {} |",
                check.name,
                trial.saturating_add(1),
                result.t_statistic,
                result.fixed_mean,
                result.random_mean,
                verdict
            )
            .map_err(|error| format!("failed to build report: {error}"))?;
        }
    }
    std::fs::write(path, report).map_err(|error| format!("failed to write {path}: {error}"))
}

fn run_trim(args: &[String]) -> Result<(), String> {
    let input = required(args, "--in")?;
    let output = required(args, "--out")?;
    let start = required_usize(args, "--start")?;
    let end = required_usize(args, "--end")?;
    let traces = read_trace_set(input)?;
    let trimmed = trim(&traces, start, end).map_err(|error| error.to_string())?;
    write_trace_set(output, &trimmed)
}

fn run_filter(args: &[String]) -> Result<(), String> {
    let input = required(args, "--in")?;
    let output = required(args, "--out")?;
    let window = required_usize(args, "--window")?;
    let traces = read_trace_set(input)?;
    let filtered = moving_average(&traces, window).map_err(|error| error.to_string())?;
    write_trace_set(output, &filtered)
}

fn run_align(args: &[String]) -> Result<(), String> {
    let input = required(args, "--in")?;
    let output = required(args, "--out")?;
    let reference = required_usize(args, "--reference")?;
    let max_shift = optional_usize(args, "--max-shift")?.unwrap_or(10);
    let traces = read_trace_set(input)?;
    let aligned = align(&traces, reference, max_shift).map_err(|error| error.to_string())?;
    write_trace_set(output, &aligned)
}

fn read_trace_set(path: &str) -> Result<TraceSet, String> {
    let file = File::open(path).map_err(|error| format!("failed to open {path}: {error}"))?;
    TraceSet::read_from(file).map_err(|error| format!("failed to read {path}: {error}"))
}

fn write_trace_set(path: &str, traces: &TraceSet) -> Result<(), String> {
    let file = File::create(path).map_err(|error| format!("failed to create {path}: {error}"))?;
    traces
        .write_to(file)
        .map_err(|error| format!("failed to write {path}: {error}"))
}

fn required<'a>(args: &'a [String], name: &str) -> Result<&'a str, String> {
    optional(args, name).ok_or_else(|| format!("missing required argument {name}"))
}

fn optional<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|argument| argument == name)
        .and_then(|index| index.checked_add(1))
        .and_then(|index| args.get(index))
        .map(String::as_str)
}

fn required_usize(args: &[String], name: &str) -> Result<usize, String> {
    parse_usize(required(args, name)?, name)
}

fn optional_usize(args: &[String], name: &str) -> Result<Option<usize>, String> {
    optional(args, name)
        .map(|value| parse_usize(value, name))
        .transpose()
}

fn required_u8(args: &[String], name: &str) -> Result<u8, String> {
    parse_u8(required(args, name)?, name)
}

fn optional_u8(args: &[String], name: &str) -> Result<Option<u8>, String> {
    optional(args, name)
        .map(|value| parse_u8(value, name))
        .transpose()
}

fn optional_u16(args: &[String], name: &str) -> Result<Option<u16>, String> {
    optional(args, name)
        .map(|value| {
            value
                .parse::<u16>()
                .map_err(|error| format!("invalid {name}: {error}"))
        })
        .transpose()
}

fn optional_i16(args: &[String], name: &str) -> Result<Option<i16>, String> {
    optional(args, name)
        .map(|value| {
            value
                .parse::<i16>()
                .map_err(|error| format!("invalid {name}: {error}"))
        })
        .transpose()
}

fn optional_u64(args: &[String], name: &str) -> Result<Option<u64>, String> {
    optional(args, name)
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|error| format!("invalid {name}: {error}"))
        })
        .transpose()
}

fn optional_f64(args: &[String], name: &str) -> Result<Option<f64>, String> {
    optional(args, name)
        .map(|value| {
            value
                .parse::<f64>()
                .map_err(|error| format!("invalid {name}: {error}"))
        })
        .transpose()
}

fn parse_usize(value: &str, name: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|error| format!("invalid {name}: {error}"))
}

fn parse_u8(value: &str, name: &str) -> Result<u8, String> {
    value.strip_prefix("0x").map_or_else(
        || {
            value
                .parse::<u8>()
                .map_err(|error| format!("invalid {name}: {error}"))
        },
        |hex| u8::from_str_radix(hex, 16).map_err(|error| format!("invalid {name}: {error}")),
    )
}

fn print_usage() {
    println!(
        "SiCaS leakage-assessment harness

Usage:
  sicas-harness tvla --fixed <file> --random <file> [--threshold 4.5]
  sicas-harness cpa --traces <file> --plaintexts <file>
  sicas-harness dpa --traces <file> --plaintexts <file>
  sicas-harness synthetic --fixed-out <file> --random-out <file> \\
      --fixed-count <n> --random-count <n> --samples <n> --secret <byte> \\
      [--fixed-plaintext <byte>] [--noise <n>] [--leakage <n>] [--seed <n>]
  sicas-harness synthetic-cpa --out <file> --plaintexts-out <file> \\
      --count <n> --samples <n> --secret <byte> \\
      [--noise <n>] [--leakage <n>] [--seed <n>]
  sicas-harness timing [--samples <n>] [--warmup <n>] [--threshold 4.5]
  sicas-harness ct [--samples <n>] [--warmup <n>] [--trials <n>] \\
      [--threshold 4.5] [--label <text>] [--report <file>]
  sicas-harness leakage-check [--samples <n>] [--seed <n>] [--threshold 0.02]
  sicas-harness trim --in <file> --out <file> --start <n> --end <n>
  sicas-harness filter --in <file> --out <file> --window <n>
  sicas-harness align --in <file> --out <file> --reference <index> [--max-shift <n>]

Commands return a non-zero exit status when leakage is detected."
    );
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

    fn arguments(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn required_and_optional_arguments_are_parsed() {
        let args = arguments(&["--value", "7"]);
        assert_eq!(optional(&args, "--value"), Some("7"));
        assert_eq!(optional(&args, "--missing"), None);
        assert_eq!(required(&args, "--value"), Ok("7"));
        assert!(required(&args, "--missing").is_err());
    }

    #[test]
    fn numeric_parsers_accept_valid_values() {
        let args = arguments(&[
            "--usize", "7", "--u8", "0x42", "--u16", "65535", "--i16", "-5", "--u64", "42",
            "--f64", "1.5",
        ]);

        assert_eq!(optional_usize(&args, "--usize"), Ok(Some(7)));
        assert_eq!(optional_u8(&args, "--u8"), Ok(Some(0x42)));
        assert_eq!(optional_u16(&args, "--u16"), Ok(Some(u16::MAX)));
        assert_eq!(optional_i16(&args, "--i16"), Ok(Some(-5)));
        assert_eq!(optional_u64(&args, "--u64"), Ok(Some(42)));
        assert_eq!(optional_f64(&args, "--f64"), Ok(Some(1.5)));
        assert_eq!(parse_usize("9", "value"), Ok(9));
        assert_eq!(parse_u8("9", "value"), Ok(9));
    }

    #[test]
    fn numeric_parsers_reject_invalid_values() {
        assert!(parse_usize("nope", "value").is_err());
        assert!(parse_u8("nope", "value").is_err());
        assert!(parse_u8("0xgg", "value").is_err());
        assert!(optional_usize(&arguments(&["--value", "nope"]), "--value").is_err());
    }

    #[test]
    fn timing_rejects_an_invalid_threshold() {
        let args = arguments(&["--samples", "10", "--warmup", "0", "--threshold", "0"]);
        assert!(run_timing(&args).is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore)] // File I/O is exercised natively.
    fn dpa_rejects_mismatched_inputs() {
        let traces = TraceSet::from_traces(vec![vec![1], vec![2]]).unwrap();
        let directory = std::env::temp_dir();
        let traces_path = directory.join(format!("sicas-dpa-traces-{}.bin", std::process::id()));
        let plaintexts_path =
            directory.join(format!("sicas-dpa-plaintexts-{}.bin", std::process::id()));
        let file = File::create(&traces_path).unwrap();
        traces.write_to(file).unwrap();
        std::fs::write(&plaintexts_path, [0_u8]).unwrap();
        let args = arguments(&[
            "--traces",
            traces_path.to_str().unwrap(),
            "--plaintexts",
            plaintexts_path.to_str().unwrap(),
        ]);

        assert!(run_dpa(&args).is_err());
        let _ = std::fs::remove_file(traces_path);
        let _ = std::fs::remove_file(plaintexts_path);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Statistical model check is exercised natively.
    fn leakage_check_runs_with_a_negative_control() {
        let args = arguments(&["--samples", "100", "--threshold", "0.5"]);
        assert!(run_leakage_check(&args).is_ok());
    }

    #[test]
    fn ct_rejects_zero_trials() {
        let args = arguments(&["--trials", "0"]);
        assert!(run_ct(&args).is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore)] // File output and timing are exercised natively.
    fn ct_writes_a_report() {
        let path = std::env::temp_dir().join(format!("sicas-ct-{}.md", std::process::id()));
        let path_text = path.to_str().unwrap();
        let args = arguments(&[
            "--samples",
            "100",
            "--warmup",
            "0",
            "--trials",
            "1",
            "--threshold",
            "1000",
            "--report",
            path_text,
        ]);

        assert!(run_ct(&args).is_ok());
        assert!(path.is_file());
        let _ = std::fs::remove_file(path);
    }
}
