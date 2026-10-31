//! Safe, zero-dependency constant-time and masking primitives for SiCaS.
//!
//! This crate provides building blocks, not complete cryptographic primitives.
//! It is `no_std`, forbids unsafe code, and has no runtime dependencies.
//!
//! # Security model
//!
//! The primitives in this crate target a first-order passive side-channel
//! adversary. They do not protect against higher-order attacks, fault
//! injection, invasive probing, or microarchitectural attacks. See the
//! project threat model and claims matrix for the complete scope.
//!
//! # Example
//!
//! ```
//! use sicas_core::{ct, Masked};
//!
//! let value = 0x1234_5678_u32;
//! let mask = 0xa5a5_a5a5_u32;
//! let masked = Masked::<u32>::mask(value, mask);
//!
//! assert_eq!(*masked.unmask().expose_secret(), value);
//! assert!(ct::eq_u32(value, value).as_u8() == 1);
//! ```

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

mod choice;
pub mod ct;
mod entropy;
mod masked;
#[cfg(kani)]
mod proofs;
mod secret;

pub use choice::Choice;
pub use entropy::{EntropyError, EntropySource};
pub use masked::Masked;
pub use secret::Secret;
