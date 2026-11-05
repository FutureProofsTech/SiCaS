//! Masked reference integrations for SiCaS.
//!
//! This crate contains validation targets, not audited production
//! cryptographic implementations. The first target is a first-order Boolean
//! masked ChaCha20 block function.

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unreachable_pub)]
#![deny(unused_must_use)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::private_intra_doc_links)]

#[cfg(test)]
extern crate std;

pub mod chacha20;
