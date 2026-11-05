//! OS-backed entropy adapter.

use std::fmt;
use std::fs::File;
use std::io::Read;

use sicas_core::{EntropyError, EntropySource};

/// An entropy source backed by the operating system.
///
/// On Unix-like systems this reads `/dev/urandom`. Other platforms fail closed
/// until a dedicated adapter is added.
pub struct OsEntropy<R = File> {
    source: R,
}

impl OsEntropy<File> {
    /// Opens the platform entropy source.
    ///
    /// # Errors
    ///
    /// Returns [`EntropyError::Unavailable`] when the platform source cannot be
    /// opened or is not supported.
    pub fn new() -> Result<Self, EntropyError> {
        #[cfg(unix)]
        {
            let source = File::open("/dev/urandom").map_err(|_| EntropyError::Unavailable)?;
            Ok(Self { source })
        }

        #[cfg(not(unix))]
        {
            Err(EntropyError::Unavailable)
        }
    }
}

impl<R> OsEntropy<R> {
    /// Wraps an existing reader as an entropy source.
    ///
    /// This is intended for tests and for platform adapters that already own a
    /// verified entropy handle.
    #[must_use]
    pub const fn from_reader(source: R) -> Self {
        Self { source }
    }
}

impl<R: Read> EntropySource for OsEntropy<R> {
    fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), EntropyError> {
        self.source
            .read_exact(destination)
            .map_err(|_| EntropyError::Unavailable)
    }
}

impl<R> fmt::Debug for OsEntropy<R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("OsEntropy<redacted>")
    }
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

    use std::io;

    use super::*;

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("entropy failure"))
        }
    }

    #[test]
    fn failing_reader_fails_closed() {
        let mut source = OsEntropy::from_reader(FailingReader);
        let mut bytes = [0_u8; 16];

        assert_eq!(
            source.fill_bytes(&mut bytes),
            Err(EntropyError::Unavailable)
        );
    }

    #[test]
    fn debug_is_redacted() {
        let source = OsEntropy::from_reader(FailingReader);
        assert_eq!(std::format!("{source:?}"), "OsEntropy<redacted>");
    }

    #[cfg(unix)]
    #[cfg_attr(miri, ignore = "Miri isolation blocks opening /dev/urandom")]
    #[test]
    fn os_entropy_fills_a_buffer() {
        let mut source = OsEntropy::new().unwrap();
        let mut bytes = [0_u8; 32];
        source.fill_bytes(&mut bytes).unwrap();

        assert!(bytes.iter().any(|byte| *byte != 0));
    }

    #[cfg(unix)]
    #[cfg_attr(miri, ignore = "Miri isolation blocks opening /dev/urandom")]
    #[test]
    fn os_entropy_produces_a_mask() {
        let mut source = OsEntropy::new().unwrap();
        assert!(source.mask_u32().is_ok());
    }
}
