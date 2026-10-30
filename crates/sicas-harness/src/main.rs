//! Command-line interface for the SiCaS leakage-assessment harness.

use std::env;
use std::fs::File;
use std::hint::black_box;
use std::process::ExitCode;

use sicas_core::ct;
use sicas_harness::cpa::cpa_hamming_weight;
use sicas_harness::synthetic::{SyntheticConfig, TvlaSyntheticConfig, generate, generate_tvla};
use sicas_harness::timing::{TimingConfig, dudect};
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
        "synthetic" => run_synthetic_tvla(command_args),
        "synthetic-cpa" => run_synthetic_cpa(command_args),
        "timing" => run_timing(command_args),
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
  sicas-harness synthetic --fixed-out <file> --random-out <file> \\
      --fixed-count <n> --random-count <n> --samples <n> --secret <byte> \\
      [--fixed-plaintext <byte>] [--noise <n>] [--leakage <n>] [--seed <n>]
  sicas-harness synthetic-cpa --out <file> --plaintexts-out <file> \\
      --count <n> --samples <n> --secret <byte> \\
      [--noise <n>] [--leakage <n>] [--seed <n>]
  sicas-harness timing [--samples <n>] [--warmup <n>] [--threshold 4.5]

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
}
