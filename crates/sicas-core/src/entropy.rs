//! Entropy-source abstraction with fail-closed behavior.

/// Error returned when secure entropy is unavailable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntropyError {
    /// The platform entropy source is unavailable or failed.
    Unavailable,
}

/// A source of cryptographically secure random bytes.
///
/// Implementations must fail closed: if secure randomness is unavailable, they
/// must return an error rather than falling back to a predictable source.
pub trait EntropySource {
    /// Fills `destination` with random bytes.
    ///
    /// # Errors
    ///
    /// Returns [`EntropyError::Unavailable`] when secure entropy cannot be
    /// obtained. Implementations must not partially fill the buffer and report
    /// success.
    fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), EntropyError>;

    /// Returns a random `u32` mask.
    ///
    /// # Errors
    ///
    /// Returns [`EntropyError::Unavailable`] when secure entropy cannot be
    /// obtained.
    #[inline]
    fn mask_u32(&mut self) -> Result<u32, EntropyError> {
        let mut bytes = [0_u8; 4];
        self.fill_bytes(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }

    /// Returns a random `u64` mask.
    ///
    /// # Errors
    ///
    /// Returns [`EntropyError::Unavailable`] when secure entropy cannot be
    /// obtained.
    #[inline]
    fn mask_u64(&mut self) -> Result<u64, EntropyError> {
        let mut bytes = [0_u8; 8];
        self.fill_bytes(&mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
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

    use super::*;

    struct FailingEntropy;

    impl EntropySource for FailingEntropy {
        fn fill_bytes(&mut self, _destination: &mut [u8]) -> Result<(), EntropyError> {
            Err(EntropyError::Unavailable)
        }
    }

    struct FixedEntropy([u8; 4]);

    impl EntropySource for FixedEntropy {
        fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), EntropyError> {
            let length = destination.len().min(self.0.len());
            destination[..length].copy_from_slice(&self.0[..length]);
            Ok(())
        }
    }

    #[test]
    fn failure_is_propagated() {
        let mut source = FailingEntropy;
        assert_eq!(source.mask_u32(), Err(EntropyError::Unavailable));
    }

    #[test]
    fn mask_is_little_endian() {
        let mut source = FixedEntropy([0x78, 0x56, 0x34, 0x12]);
        assert_eq!(source.mask_u32(), Ok(0x1234_5678));

        let mut source = FixedEntropy([0x78, 0x56, 0x34, 0x12]);
        assert_eq!(source.mask_u64(), Ok(0x1234_5678));
    }
}
