//! Temporal errors.

use thiserror::Error;

/// An error expressing when a timestamp is outside the representable range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum OutOfRangeError {
    /// The timestamp exceeds 2⁵³ seconds past the Unix epoch.
    #[error("Time exceeds JsTime (2⁵³) range: {0}")]
    TooLarge(u64),

    /// The timestamp is before the Unix epoch.
    #[error("Time is before the Unix epoch")]
    BeforeEpoch,
}

/// An error expressing when a time is larger than 2⁵³ seconds past the Unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum NumberIsNotATimestamp {
    /// The [`Ipld`] number that is outside of the [`JsTime`] range.
    #[error("Cannot convert IPLD number to JsTime (2⁵³) range: {0}")]
    TriedIpldInt(i128),

    /// A Unix seconds value is outside of the safe range.
    #[error(transparent)]
    TriedU64(#[from] OutOfRangeError),
}

/// An error expressing when a time is not within the bounds of a UCAN.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum TimeBoundError {
    /// The UCAN has expired.
    #[error("Expired")]
    Expired,

    /// The UCAN is not yet valid, but will be in the future.
    #[error("Not yet valid")]
    NotYetValid,
}

/// The UCAN has expired.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Error)]
#[error("Expired")]
pub struct Expired;
