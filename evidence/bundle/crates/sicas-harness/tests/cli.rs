//! End-to-end tests for the `sicas-harness` command-line interface.
//!
//! Miri cannot execute subprocesses under isolation, so this integration test
//! binary is skipped under Miri. The CLI logic is still covered by unit tests.

#![cfg(not(miri))]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::integer_division,
    clippy::as_conversions
)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

const fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_sicas-harness")
}

fn temporary_directory(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("sicas-harness-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn run(arguments: &[&str]) -> std::process::Output {
    Command::new(binary())
        .args(arguments)
        .output()
        .expect("failed to run sicas-harness")
}

#[test]
fn help_is_available() {
    let output = run(&["--help"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage"));
}

#[test]
fn unknown_command_fails() {
    let output = run(&["not-a-command"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown command"));
}

#[test]
fn missing_required_argument_fails() {
    let output = run(&["tvla"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing required argument"));
}

#[test]
fn synthetic_tvla_and_cpa_workflows_succeed() {
    let directory = temporary_directory("workflow");
    let fixed = directory.join("fixed.bin");
    let random = directory.join("random.bin");
    let traces = directory.join("traces.bin");
    let plaintexts = directory.join("plaintexts.bin");

    let synthetic = run(&[
        "synthetic",
        "--fixed-out",
        fixed.to_str().unwrap(),
        "--random-out",
        random.to_str().unwrap(),
        "--fixed-count",
        "128",
        "--random-count",
        "128",
        "--samples",
        "4",
        "--secret",
        "0x42",
        "--noise",
        "10",
        "--leakage",
        "100",
        "--seed",
        "11",
    ]);
    assert!(synthetic.status.success());
    assert!(fixed.is_file());
    assert!(random.is_file());

    let tvla = run(&[
        "tvla",
        "--fixed",
        fixed.to_str().unwrap(),
        "--random",
        random.to_str().unwrap(),
    ]);
    assert!(!tvla.status.success());
    assert!(String::from_utf8_lossy(&tvla.stdout).contains("TVLA"));
    assert!(String::from_utf8_lossy(&tvla.stderr).contains("TVLA detected leakage"));

    let synthetic_cpa = run(&[
        "synthetic-cpa",
        "--out",
        traces.to_str().unwrap(),
        "--plaintexts-out",
        plaintexts.to_str().unwrap(),
        "--count",
        "128",
        "--samples",
        "4",
        "--secret",
        "0x42",
        "--noise",
        "0",
        "--leakage",
        "100",
        "--seed",
        "7",
    ]);
    assert!(synthetic_cpa.status.success());
    assert!(traces.is_file());
    assert!(plaintexts.is_file());

    let cpa = run(&[
        "cpa",
        "--traces",
        traces.to_str().unwrap(),
        "--plaintexts",
        plaintexts.to_str().unwrap(),
    ]);
    assert!(cpa.status.success());
    assert!(String::from_utf8_lossy(&cpa.stdout).contains("0x42"));

    let _ = fs::remove_dir_all(directory);
}

#[test]
fn preprocessing_workflow_succeeds() {
    let directory = temporary_directory("preprocess");
    let traces = directory.join("traces.bin");
    let plaintexts = directory.join("plaintexts.bin");
    let trimmed = directory.join("trimmed.bin");
    let filtered = directory.join("filtered.bin");
    let aligned = directory.join("aligned.bin");

    let synthetic = run(&[
        "synthetic-cpa",
        "--out",
        traces.to_str().unwrap(),
        "--plaintexts-out",
        plaintexts.to_str().unwrap(),
        "--count",
        "16",
        "--samples",
        "8",
        "--secret",
        "0x42",
        "--noise",
        "0",
        "--leakage",
        "100",
        "--seed",
        "3",
    ]);
    assert!(synthetic.status.success());

    let trim = run(&[
        "trim",
        "--in",
        traces.to_str().unwrap(),
        "--out",
        trimmed.to_str().unwrap(),
        "--start",
        "0",
        "--end",
        "4",
    ]);
    assert!(trim.status.success());
    assert!(trimmed.is_file());

    let filter = run(&[
        "filter",
        "--in",
        trimmed.to_str().unwrap(),
        "--out",
        filtered.to_str().unwrap(),
        "--window",
        "3",
    ]);
    assert!(filter.status.success());
    assert!(filtered.is_file());

    let align = run(&[
        "align",
        "--in",
        filtered.to_str().unwrap(),
        "--out",
        aligned.to_str().unwrap(),
        "--reference",
        "0",
        "--max-shift",
        "2",
    ]);
    assert!(align.status.success());
    assert!(aligned.is_file());

    let _ = fs::remove_dir_all(directory);
}
