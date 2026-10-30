//! Trace storage and binary input/output.
//!
//! The binary format is intentionally small and strict:
//!
//! ```text
//! magic:            8 bytes, b"SiCaSTR1"
//! version:          u16 little-endian, currently 1
//! samples_per_trace:u32 little-endian
//! trace_count:      u32 little-endian
//! samples:          trace_count * samples_per_trace i16 little-endian
//! ```

use std::fmt;
use std::io::{Read, Write};

/// Magic bytes for the SiCaS trace format.
pub const MAGIC: [u8; 8] = *b"SiCaSTR1";
/// Current trace-format version.
pub const VERSION: u16 = 1;
/// Maximum number of traces accepted by the reader.
pub const MAX_TRACES: usize = 1_000_000;
/// Maximum number of samples per trace accepted by the reader.
pub const MAX_SAMPLES: usize = 1_000_000;
/// Maximum total number of samples accepted by the reader.
pub const MAX_TOTAL_SAMPLES: usize = 100_000_000;

/// Errors produced by trace parsing, validation, and writing.
#[derive(Debug)]
pub enum TraceError {
    /// An underlying I/O operation failed.
    Io(std::io::Error),
    /// The input does not start with [`MAGIC`].
    InvalidMagic,
    /// The input uses an unsupported format version.
    UnsupportedVersion(u16),
    /// The trace dimensions are zero or inconsistent.
    InvalidDimensions,
    /// The trace set exceeds a configured size limit.
    TooLarge,
    /// The number of samples does not match the declared dimensions.
    LengthMismatch,
    /// A split index is outside the trace set.
    InvalidIndex,
}

impl fmt::Display for TraceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::InvalidMagic => formatter.write_str("invalid trace magic"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported trace version: {version}")
            }
            Self::InvalidDimensions => formatter.write_str("invalid trace dimensions"),
            Self::TooLarge => formatter.write_str("trace set exceeds a size limit"),
            Self::LengthMismatch => formatter.write_str("trace lengths do not match"),
            Self::InvalidIndex => formatter.write_str("trace split index is out of range"),
        }
    }
}

impl std::error::Error for TraceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for TraceError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// A rectangular set of power or EM traces.
#[derive(Clone)]
pub struct TraceSet {
    samples_per_trace: usize,
    traces: Vec<Vec<i16>>,
}

impl fmt::Debug for TraceSet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TraceSet")
            .field("traces", &self.traces.len())
            .field("samples_per_trace", &self.samples_per_trace)
            .finish()
    }
}

impl TraceSet {
    /// Builds a trace set from owned traces.
    ///
    /// # Errors
    ///
    /// Returns an error if the set is empty, a trace is empty, the traces have
    /// different lengths, or the set exceeds a size limit.
    pub fn from_traces(traces: Vec<Vec<i16>>) -> Result<Self, TraceError> {
        let Some(first) = traces.first() else {
            return Err(TraceError::InvalidDimensions);
        };
        let samples_per_trace = first.len();
        validate_dimensions(samples_per_trace, traces.len())?;

        if traces.iter().any(|trace| trace.len() != samples_per_trace) {
            return Err(TraceError::LengthMismatch);
        }

        Ok(Self {
            samples_per_trace,
            traces,
        })
    }

    /// Returns the number of traces.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.traces.len()
    }

    /// Returns `true` when the set contains no traces.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.traces.is_empty()
    }

    /// Returns the number of samples in each trace.
    #[must_use]
    pub const fn samples_per_trace(&self) -> usize {
        self.samples_per_trace
    }

    /// Returns one trace by index.
    #[must_use]
    pub fn trace(&self, index: usize) -> Option<&[i16]> {
        self.traces.get(index).map(Vec::as_slice)
    }

    /// Returns all traces.
    #[must_use]
    pub fn traces(&self) -> &[Vec<i16>] {
        &self.traces
    }

    /// Splits the trace set at `index`.
    ///
    /// Either side may be empty. Empty sides are useful for analysis but cannot
    /// be serialized by [`TraceSet::write_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when `index` is greater than the number of traces.
    pub fn split_at(&self, index: usize) -> Result<(Self, Self), TraceError> {
        if index > self.traces.len() {
            return Err(TraceError::InvalidIndex);
        }

        let (left, right) = self.traces.split_at(index);
        Ok((
            Self {
                samples_per_trace: self.samples_per_trace,
                traces: left.to_vec(),
            },
            Self {
                samples_per_trace: self.samples_per_trace,
                traces: right.to_vec(),
            },
        ))
    }

    /// Reads a trace set from the binary format.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid magic, unsupported versions, malformed
    /// dimensions, truncated input, or size-limit violations.
    pub fn read_from<R: Read>(mut reader: R) -> Result<Self, TraceError> {
        let mut magic = [0_u8; 8];
        reader.read_exact(&mut magic)?;
        if magic != MAGIC {
            return Err(TraceError::InvalidMagic);
        }

        let version = read_u16(&mut reader)?;
        if version != VERSION {
            return Err(TraceError::UnsupportedVersion(version));
        }

        let samples_per_trace =
            usize::try_from(read_u32(&mut reader)?).map_err(|_| TraceError::TooLarge)?;
        let trace_count =
            usize::try_from(read_u32(&mut reader)?).map_err(|_| TraceError::TooLarge)?;
        validate_dimensions(samples_per_trace, trace_count)?;

        let mut traces = Vec::new();
        traces
            .try_reserve_exact(trace_count)
            .map_err(|_| TraceError::TooLarge)?;

        let mut bytes = [0_u8; 2];
        for _ in 0..trace_count {
            let mut trace = Vec::new();
            trace
                .try_reserve_exact(samples_per_trace)
                .map_err(|_| TraceError::TooLarge)?;

            for _ in 0..samples_per_trace {
                reader.read_exact(&mut bytes)?;
                trace.push(i16::from_le_bytes(bytes));
            }

            traces.push(trace);
        }

        Ok(Self {
            samples_per_trace,
            traces,
        })
    }

    /// Writes the trace set in the binary format.
    ///
    /// # Errors
    ///
    /// Returns an error if the dimensions exceed the format limits or the
    /// underlying writer fails.
    pub fn write_to<W: Write>(&self, mut writer: W) -> Result<(), TraceError> {
        if self.traces.is_empty() {
            return Err(TraceError::InvalidDimensions);
        }

        let samples_per_trace =
            u32::try_from(self.samples_per_trace).map_err(|_| TraceError::TooLarge)?;
        let trace_count = u32::try_from(self.traces.len()).map_err(|_| TraceError::TooLarge)?;

        writer.write_all(&MAGIC)?;
        writer.write_all(&VERSION.to_le_bytes())?;
        writer.write_all(&samples_per_trace.to_le_bytes())?;
        writer.write_all(&trace_count.to_le_bytes())?;

        for trace in &self.traces {
            for sample in trace {
                writer.write_all(&sample.to_le_bytes())?;
            }
        }

        Ok(())
    }
}

fn validate_dimensions(samples_per_trace: usize, trace_count: usize) -> Result<(), TraceError> {
    if samples_per_trace == 0 || trace_count == 0 {
        return Err(TraceError::InvalidDimensions);
    }
    if samples_per_trace > MAX_SAMPLES || trace_count > MAX_TRACES {
        return Err(TraceError::TooLarge);
    }
    let total_samples = samples_per_trace
        .checked_mul(trace_count)
        .ok_or(TraceError::TooLarge)?;
    if total_samples > MAX_TOTAL_SAMPLES {
        return Err(TraceError::TooLarge);
    }
    Ok(())
}

fn read_u16<R: Read>(reader: &mut R) -> Result<u16, TraceError> {
    let mut bytes = [0_u8; 2];
    reader.read_exact(&mut bytes)?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_u32<R: Read>(reader: &mut R) -> Result<u32, TraceError> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
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

    fn sample_set() -> TraceSet {
        TraceSet::from_traces(vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap()
    }

    #[test]
    fn round_trip_preserves_traces() {
        let traces = sample_set();
        let mut bytes = Vec::new();
        traces.write_to(&mut bytes).unwrap();

        let decoded = TraceSet::read_from(&bytes[..]).unwrap();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded.samples_per_trace(), 3);
        assert_eq!(decoded.trace(0), Some([1, 2, 3].as_slice()));
        assert_eq!(decoded.trace(1), Some([4, 5, 6].as_slice()));
    }

    #[test]
    fn invalid_magic_is_rejected() {
        let mut bytes = Vec::new();
        sample_set().write_to(&mut bytes).unwrap();
        bytes[0] ^= 0xff;

        assert!(matches!(
            TraceSet::read_from(&bytes[..]),
            Err(TraceError::InvalidMagic)
        ));
    }

    #[test]
    fn ragged_traces_are_rejected() {
        let result = TraceSet::from_traces(vec![vec![1, 2], vec![3]]);
        assert!(matches!(result, Err(TraceError::LengthMismatch)));
    }

    #[test]
    fn empty_trace_sets_are_rejected() {
        let result = TraceSet::from_traces(Vec::new());
        assert!(matches!(result, Err(TraceError::InvalidDimensions)));
    }

    #[test]
    fn split_preserves_dimensions() {
        let (left, right) = sample_set().split_at(1).unwrap();
        assert_eq!(left.len(), 1);
        assert_eq!(right.len(), 1);
        assert_eq!(left.samples_per_trace(), 3);
        assert_eq!(right.samples_per_trace(), 3);
    }

    #[test]
    fn split_out_of_range_is_rejected() {
        assert!(matches!(
            sample_set().split_at(3),
            Err(TraceError::InvalidIndex)
        ));
    }

    #[test]
    fn split_at_end_produces_an_empty_right_side() {
        let (left, right) = sample_set().split_at(2).unwrap();
        assert_eq!(left.len(), 2);
        assert!(right.is_empty());

        let mut bytes = Vec::new();
        assert!(matches!(
            right.write_to(&mut bytes),
            Err(TraceError::InvalidDimensions)
        ));
    }

    #[test]
    fn unsupported_version_is_rejected() {
        let mut bytes = Vec::new();
        sample_set().write_to(&mut bytes).unwrap();
        bytes[8..10].copy_from_slice(&2_u16.to_le_bytes());

        assert!(matches!(
            TraceSet::read_from(&bytes[..]),
            Err(TraceError::UnsupportedVersion(2))
        ));
    }

    #[test]
    fn zero_dimensions_are_rejected() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC);
        bytes.extend_from_slice(&VERSION.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u32.to_le_bytes());

        assert!(matches!(
            TraceSet::read_from(&bytes[..]),
            Err(TraceError::InvalidDimensions)
        ));
    }

    #[test]
    fn dimension_limits_are_enforced() {
        assert!(validate_dimensions(MAX_SAMPLES, 1).is_ok());
        assert!(validate_dimensions(1, MAX_TRACES).is_ok());
        assert!(validate_dimensions(MAX_SAMPLES + 1, 1).is_err());
        assert!(validate_dimensions(1, MAX_TRACES + 1).is_err());
        assert!(validate_dimensions(0, 1).is_err());
        assert!(validate_dimensions(1, 0).is_err());
        assert!(validate_dimensions(MAX_SAMPLES, MAX_TOTAL_SAMPLES / MAX_SAMPLES).is_ok());
        assert!(validate_dimensions(MAX_SAMPLES, (MAX_TOTAL_SAMPLES / MAX_SAMPLES) + 1).is_err());
    }

    #[test]
    fn debug_is_a_summary() {
        let rendered = std::format!("{:?}", sample_set());
        assert!(rendered.contains("TraceSet"));
        assert!(rendered.contains("traces"));
    }

    #[test]
    fn error_display_and_source_are_populated() {
        let error = TraceError::Io(std::io::Error::other("boom"));
        assert!(error.to_string().contains("I/O"));
        assert!(std::error::Error::source(&error).is_some());

        assert!(TraceError::InvalidMagic.to_string().contains("magic"));
        assert!(TraceError::UnsupportedVersion(9).to_string().contains('9'));
        assert!(
            TraceError::InvalidDimensions
                .to_string()
                .contains("dimensions")
        );
        assert!(TraceError::TooLarge.to_string().contains("size"));
        assert!(TraceError::LengthMismatch.to_string().contains("length"));
        assert!(TraceError::InvalidIndex.to_string().contains("index"));
    }
}
