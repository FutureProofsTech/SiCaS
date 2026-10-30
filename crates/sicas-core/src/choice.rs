//! Constant-time boolean choices.

/// A boolean value represented as `0` or `1`.
///
/// `Choice` is intended for results of constant-time comparisons. It is not a
/// secret container, and callers must not use it to branch on secret data.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[must_use]
pub struct Choice(u8);

impl Choice {
    /// Creates a choice from a public boolean.
    pub fn from_bool(value: bool) -> Self {
        Self(u8::from(value))
    }

    /// Returns the underlying `0` or `1`.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }

    /// Returns an all-ones byte mask when true, or zero when false.
    #[must_use]
    pub const fn mask_u8(self) -> u8 {
        self.0.wrapping_neg()
    }

    /// Returns an all-ones `u16` mask when true, or zero when false.
    #[must_use]
    pub const fn mask_u16(self) -> u16 {
        let byte = self.0.wrapping_neg();
        u16::from_le_bytes([byte, byte])
    }

    /// Returns an all-ones `u32` mask when true, or zero when false.
    #[must_use]
    pub const fn mask_u32(self) -> u32 {
        let byte = self.0.wrapping_neg();
        u32::from_le_bytes([byte; 4])
    }

    /// Returns an all-ones `u64` mask when true, or zero when false.
    #[must_use]
    pub const fn mask_u64(self) -> u64 {
        let byte = self.0.wrapping_neg();
        u64::from_le_bytes([byte; 8])
    }

    /// Returns an all-ones `u128` mask when true, or zero when false.
    #[must_use]
    pub const fn mask_u128(self) -> u128 {
        let byte = self.0.wrapping_neg();
        u128::from_le_bytes([byte; 16])
    }

    pub(crate) const fn from_raw_u8(value: u8) -> Self {
        Self(value)
    }

    pub(crate) const fn from_raw_u16(value: u16) -> Self {
        let [low, _] = value.to_le_bytes();
        Self(low)
    }

    pub(crate) const fn from_raw_u32(value: u32) -> Self {
        let [low, _, _, _] = value.to_le_bytes();
        Self(low)
    }

    pub(crate) const fn from_raw_u64(value: u64) -> Self {
        let [low, _, _, _, _, _, _, _] = value.to_le_bytes();
        Self(low)
    }

    pub(crate) const fn from_raw_u128(value: u128) -> Self {
        let [low, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] = value.to_le_bytes();
        Self(low)
    }
}
