//! Zero-dependency leakage-assessment harness for SiCaS.
//!
//! The harness reads power/EM trace sets, runs fixed-vs-random TVLA, performs a
//! byte-oriented correlation power analysis, generates deterministic synthetic
//! traces for validation, and provides dudect-style timing checks.
//!
//! This crate is analysis tooling. It does not make security claims on its own;
//! results must be published with raw data, parameters, and reproduction steps.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unreachable_pub)]
#![deny(unused_must_use)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::private_intra_doc_links)]

pub mod cpa;
pub mod dpa;
pub mod leakage;
pub mod preprocess;
pub mod stats;
pub mod synthetic;
pub mod timing;
pub mod trace;
pub mod tvla;
