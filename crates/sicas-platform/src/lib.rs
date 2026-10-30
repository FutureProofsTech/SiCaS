//! Safe OS platform adapters for SiCaS.
//!
//! This crate uses only safe Rust and std APIs. It provides a fail-closed entropy
//! adapter; platforms without a supported source return an error rather than
//! falling back to a weak source.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unreachable_pub)]
#![deny(unused_must_use)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::private_intra_doc_links)]

pub mod entropy;
