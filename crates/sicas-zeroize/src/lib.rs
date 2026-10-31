//! Experimental volatile zeroization boundary.
//!
//! This crate is **not** audited, is not used by the published crates, and must
//! not be treated as a production zeroization guarantee. It exists to make the
//! unsafe boundary small, explicit, and testable before independent review.
//!
//! The implementation uses volatile writes and a compiler fence. It cannot
//! guarantee that copies in registers, temporaries, or moved values are erased.

#![no_std]
#![allow(unsafe_code)] // One isolated volatile-write boundary; reviewed below.
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unreachable_pub)]
#![deny(unused_must_use)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::private_intra_doc_links)]

#[cfg(test)]
extern crate std;

use core::fmt;
use core::sync::atomic::{Ordering, compiler_fence};

#[cfg(test)]
static DROPPED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

/// Overwrites every byte with a volatile zero write, then emits a compiler fence.
///
/// This is a best-effort erasure primitive. It does not cover copies that the
/// compiler or caller has already made elsewhere.
pub fn zeroize_bytes(buffer: &mut [u8]) {
    for byte in buffer {
        // SAFETY: `byte` is a valid mutable reference for the duration of the
        // loop. A volatile write prevents the store from being optimized away.
        unsafe {
            core::ptr::write_volatile(byte, 0_u8);
        }
    }
    compiler_fence(Ordering::SeqCst);
}

/// A fixed-size byte buffer that zeroizes its contents on drop.
///
/// The contents are exposed only through explicit [`Self::as_bytes`] and
/// [`Self::as_mut_bytes`] calls.
#[must_use]
pub struct ZeroizingBytes<const N: usize>([u8; N]);

impl<const N: usize> ZeroizingBytes<N> {
    /// Wraps a byte array.
    pub const fn new(bytes: [u8; N]) -> Self {
        Self(bytes)
    }

    /// Returns the wrapped bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; N] {
        &self.0
    }

    /// Returns mutable access to the wrapped bytes.
    pub const fn as_mut_bytes(&mut self) -> &mut [u8; N] {
        &mut self.0
    }
}

impl<const N: usize> Drop for ZeroizingBytes<N> {
    fn drop(&mut self) {
        zeroize_bytes(&mut self.0);
        #[cfg(test)]
        DROPPED.store(true, Ordering::SeqCst);
    }
}

impl<const N: usize> fmt::Debug for ZeroizingBytes<N> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ZeroizingBytes<redacted>")
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )]

    use super::*;

    #[test]
    fn zeroize_bytes_clears_the_buffer() {
        let mut buffer = [0xaa_u8; 16];
        zeroize_bytes(&mut buffer);
        assert!(buffer.iter().all(|byte| *byte == 0));
    }

    #[test]
    fn zeroizing_bytes_exposes_contents_explicitly() {
        let mut buffer = ZeroizingBytes::new([1_u8, 2, 3]);
        assert_eq!(buffer.as_bytes(), &[1, 2, 3]);
        buffer.as_mut_bytes()[0] = 9;
        assert_eq!(buffer.as_bytes(), &[9, 2, 3]);
    }

    #[test]
    fn drop_runs_the_zeroization_path() {
        DROPPED.store(false, Ordering::SeqCst);
        {
            let _buffer = ZeroizingBytes::new([0xaa_u8; 8]);
        }
        assert!(DROPPED.load(Ordering::SeqCst));
    }

    #[test]
    fn debug_is_redacted() {
        let buffer = ZeroizingBytes::new([1_u8, 2, 3]);
        assert_eq!(std::format!("{buffer:?}"), "ZeroizingBytes<redacted>");
    }
}
