//! A timestamp type with safe JavaScript interop.

use super::error::{NumberIsNotATimestamp, OutOfRangeError};
use ipld_core::ipld::Ipld;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[cfg(feature = "std")]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(any(test, feature = "test_utils"))]
use arbitrary::{self, Arbitrary, Unstructured};

/// A [`Timestamp`] with safe JavaScript interop.
///
/// Per the UCAN spec, timestamps MUST respect [IEEE-754]
/// (64-bit double precision = 53-bit truncated integer) for
/// JavaScript interoperability.
///
/// This range can represent millions of years into the future,
/// and is thus sufficient for "nearly" all auth use cases.
///
/// Internally stores Unix seconds as a [`u64`].
///
/// [IEEE-754]: https://en.wikipedia.org/wiki/IEEE_754
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Create a [`Timestamp`] from Unix epoch seconds.
    ///
    /// # Arguments
    ///
    /// * `secs` — Seconds since the Unix epoch
    ///
    /// # Errors
    ///
    /// * [`OutOfRangeError`] — If the time is more than 2⁵³ seconds since the Unix epoch
    pub const fn from_unix(secs: u64) -> Result<Self, OutOfRangeError> {
        if secs > 0x001F_FFFF_FFFF_FFFF {
            Err(OutOfRangeError::TooLarge(secs))
        } else {
            Ok(Timestamp(secs))
        }
    }

    /// Create a [`Timestamp`] from a [`SystemTime`].
    ///
    /// # Arguments
    ///
    /// * `time` — The time to convert
    ///
    /// # Errors
    ///
    /// * [`OutOfRangeError`] — If the time is more than 2⁵³ seconds since the Unix epoch
    #[cfg(feature = "std")]
    pub fn new(time: SystemTime) -> Result<Self, OutOfRangeError> {
        let secs = time
            .duration_since(UNIX_EPOCH)
            .map_err(|_| OutOfRangeError::BeforeEpoch)?
            .as_secs();
        Self::from_unix(secs)
    }

    /// Get the current time in seconds since [`UNIX_EPOCH`] as a [`Timestamp`].
    ///
    /// # Panics
    ///
    /// This function will panic if the current time is before the Unix epoch.
    #[cfg(feature = "std")]
    #[must_use]
    #[allow(clippy::expect_used)]
    pub fn now() -> Timestamp {
        Self::new(SystemTime::now())
            .expect("the current time to be sometime in the 3rd millennium CE")
    }

    /// Get a timestamp 5 minutes from now.
    ///
    /// # Panics
    ///
    /// This function will panic if the current time is before the Unix epoch.
    #[cfg(feature = "std")]
    #[must_use]
    #[allow(clippy::expect_used)]
    pub fn five_minutes_from_now() -> Timestamp {
        Self::new(SystemTime::now() + Duration::from_secs(5 * 60))
            .expect("the current time to be sometime in the 3rd millennium CE")
    }

    /// Get a timestamp 5 years from now.
    ///
    /// # Panics
    ///
    /// This function will panic if the current time is before the Unix epoch.
    #[cfg(feature = "std")]
    #[must_use]
    #[allow(clippy::expect_used)]
    pub fn five_years_from_now() -> Timestamp {
        Self::new(SystemTime::now() + Duration::from_secs(5 * 365 * 24 * 60 * 60))
            .expect("the current time to be sometime in the 3rd millennium CE")
    }

    /// Convert a [`Timestamp`] to a [Unix timestamp].
    ///
    /// [Unix timestamp]: https://en.wikipedia.org/wiki/Unix_time
    #[must_use]
    pub const fn to_unix(&self) -> u64 {
        self.0
    }

    /// An intentionally permissive constructor from Unix seconds for
    /// deserialization. Skips the 2⁵³ bound check (Postel's law).
    pub(crate) const fn postel_unix(secs: u64) -> Self {
        Timestamp(secs)
    }
}

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
impl Timestamp {
    /// Lift a [`js_sys::Date`] into a Rust [`Timestamp`].
    ///
    /// # Errors
    ///
    /// Returns [`OutOfRangeError`] if the date exceeds the 2⁵³ second bound.
    pub fn from_date(date_time: &js_sys::Date) -> Result<Timestamp, OutOfRangeError> {
        let millis = date_time.get_time();
        if millis < 0.0 {
            return Err(OutOfRangeError::BeforeEpoch);
        }
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let secs = (millis / 1000.0) as u64;
        Timestamp::from_unix(secs)
    }

    /// Lower the [`Timestamp`] to a [`js_sys::Date`].
    ///
    /// # Errors
    ///
    /// Returns [`OutOfRangeError`] if the timestamp in _milliseconds_
    /// would exceed the 2⁵³ safe-integer bound for JavaScript `Date`.
    pub fn to_date(&self) -> Result<js_sys::Date, OutOfRangeError> {
        // 2⁵³ / 1000 = max seconds that fit in a JS Date without precision loss
        const MAX_SAFE_SECS: u64 = 0x001F_FFFF_FFFF_FFFF / 1000;
        if self.0 > MAX_SAFE_SECS {
            return Err(OutOfRangeError::TooLarge(self.0));
        }
        #[allow(clippy::cast_precision_loss)]
        let millis = self.0 as f64 * 1000.0;
        Ok(js_sys::Date::new(&wasm_bindgen::JsValue::from(millis)))
    }
}

#[cfg(feature = "std")]
impl TryFrom<SystemTime> for Timestamp {
    type Error = OutOfRangeError;

    fn try_from(sys_time: SystemTime) -> Result<Timestamp, Self::Error> {
        Timestamp::new(sys_time)
    }
}

#[cfg(feature = "std")]
impl From<Timestamp> for SystemTime {
    fn from(ts: Timestamp) -> Self {
        UNIX_EPOCH + Duration::from_secs(ts.0)
    }
}

impl From<Timestamp> for Ipld {
    fn from(timestamp: Timestamp) -> Self {
        timestamp.to_unix().into()
    }
}

impl TryFrom<Ipld> for Timestamp {
    type Error = TimestampFromIpldError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Integer(secs) => secs
                .try_into()
                .map_err(TimestampFromIpldError::NotATimestamp),
            Ipld::Null
            | Ipld::Bool(_)
            | Ipld::Float(_)
            | Ipld::String(_)
            | Ipld::Bytes(_)
            | Ipld::List(_)
            | Ipld::Map(_)
            | Ipld::Link(_) => Err(TimestampFromIpldError::TimestampIsNotAnInteger),
        }
    }
}

/// Errors when an IPLD value is not a valid timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TimestampFromIpldError {
    /// The IPLD value was not an integer.
    #[error("the timestamp is not an integer")]
    TimestampIsNotAnInteger,

    /// The integer could not be converted to a timestamp.
    #[error("the timestamp is out of bounds (2^53)")]
    NotATimestamp(#[from] NumberIsNotATimestamp),
}

impl From<Timestamp> for i128 {
    fn from(timestamp: Timestamp) -> i128 {
        i128::from(timestamp.to_unix())
    }
}

impl TryFrom<i128> for Timestamp {
    type Error = NumberIsNotATimestamp;

    fn try_from(secs: i128) -> Result<Self, Self::Error> {
        let secs_u64 =
            u64::try_from(secs).map_err(|_| NumberIsNotATimestamp::TriedIpldInt(secs))?;
        Ok(Timestamp::from_unix(secs_u64)?)
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = u64::deserialize(deserializer)?;
        Ok(Timestamp::postel_unix(seconds))
    }
}

#[cfg(any(test, feature = "test_utils"))]
impl<'a> Arbitrary<'a> for Timestamp {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, arbitrary::Error> {
        let secs = u.int_in_range(core::ops::RangeInclusive::new(0, u64::pow(2, 53) - 1))?;
        Ok(Timestamp(secs))
    }
}
