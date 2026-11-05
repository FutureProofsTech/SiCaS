//! Opaque secret values.

use core::fmt;

/// A value that must not be exposed accidentally.
///
/// `Secret` deliberately does not implement `PartialEq`, `Ord`, `Hash`,
/// `Index`, `Clone`, or `Copy`. Use explicit constant-time operations and call
/// [`Secret::expose_secret`] only at a documented declassification boundary.
///
/// # Zeroization
///
/// This type does not guarantee that the inner value is erased from memory when
/// dropped. Production deployments must use a platform adapter that provides
/// verified zeroization.
///
/// # Example
///
/// ```
/// use sicas_core::Secret;
///
/// let secret = Secret::new(0xdead_beef_u32);
/// assert_eq!(*secret.expose_secret(), 0xdead_beef);
/// assert_eq!(format!("{secret:?}"), "Secret<redacted>");
/// ```
///
/// Equality is intentionally unavailable:
///
/// ```compile_fail
/// use sicas_core::Secret;
///
/// let a = Secret::new(1_u32);
/// let b = Secret::new(1_u32);
/// assert_eq!(a, b);
/// ```
///
/// Indexing is intentionally unavailable:
///
/// ```compile_fail
/// use sicas_core::Secret;
///
/// let secret = Secret::new([1_u8, 2_u8]);
/// let _ = secret[0];
/// ```
///
/// Cloning is intentionally unavailable:
///
/// ```compile_fail
/// use sicas_core::Secret;
///
/// let secret = Secret::new(1_u32);
/// let _copy = secret.clone();
/// ```
#[must_use]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    /// Wraps a value as a secret.
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// Explicitly declassifies the secret.
    ///
    /// Every call is a declassification boundary and must be justified in the
    /// caller's leakage contract.
    #[must_use]
    pub const fn expose_secret(&self) -> &T {
        &self.0
    }

    /// Consumes the wrapper and returns the inner value.
    ///
    /// This is a declassification boundary.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Secret<redacted>")
    }
}
