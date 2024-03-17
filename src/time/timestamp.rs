//! A JavaScript-wrapper for [`Timestamp`][crate::time::Timestamp].

use super::OutOfRangeError;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use web_time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

/// A [`Timestamp`][super::Timestamp] with safe JavaScript interop.
///
/// Per the UCAN spec, timestamps MUST respect [IEEE-754]
/// (64-bit double precision = 53-bit truncated integer) for
/// JavaScript interoperability.
///
/// This range can represent millions of years into the future,
/// and is thus sufficient for "nearly" all auth use cases.
///
/// This type internally deserializes permissively from any [`SystemTime`],
/// but checks that any time created is in the 53-bit bound when created via
/// the public API.
///
/// [IEEE-754]: https://en.wikipedia.org/wiki/IEEE_754
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Timestamp {
    time: SystemTime,
}

impl Timestamp {
    /// Create a [`Timestamp`] from a [`SystemTime`].
    ///
    /// # Arguments
    ///
    /// * `time` — The time to convert
    ///
    /// # Errors
    ///
    /// * [`OutOfRangeError`] — If the time is more than 2⁵³ seconds since the Unix epoch
    pub fn new(time: SystemTime) -> Result<Self, OutOfRangeError> {
        if time
            .duration_since(UNIX_EPOCH)
            .map_err(|_| OutOfRangeError { tried: time })?
            .as_secs()
            > 0x1FFFFFFFFFFFFF
        {
            Err(OutOfRangeError { tried: time })
        } else {
            Ok(Timestamp { time })
        }
    }

    /// Get the current time in seconds since [`UNIX_EPOCH`] as a [`Timestamp`].
    pub fn now() -> Timestamp {
        Self::new(SystemTime::now())
            .expect("the current time to be somtime in the 3rd millenium CE")
    }

    pub fn five_minutes_from_now() -> Timestamp {
        Self::new(SystemTime::now() + Duration::from_secs(5 * 60))
            .expect("the current time to be somtime in the 3rd millenium CE")
    }

    pub fn five_years_from_now() -> Timestamp {
        Self::new(SystemTime::now() + Duration::from_secs(5 * 365 * 24 * 60 * 60))
            .expect("the current time to be somtime in the 3rd millenium CE")
    }

    /// Convert a [`Timestamp`] to a [Unix timestamp].
    ///
    /// [Unix timestamp]: https://en.wikipedia.org/wiki/Unix_time
    pub fn to_unix(&self) -> u64 {
        self.time
            .duration_since(UNIX_EPOCH)
            .expect("System time to be after the Unix epoch")
            .as_secs()
    }

    /// An intentionally permissive variant of `new` for
    /// deseriazation. See the note on the struct.
    pub(crate) fn postel(time: SystemTime) -> Self {
        Timestamp { time }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl Timestamp {
    /// Lift a [`js_sys::Date`] into a Rust [`Timestamp`]
    pub fn from_date(date_time: js_sys::Date) -> Result<Timestamp, JsError> {
        let millis = date_time.get_time() as u64;
        let secs: u64 = (millis / 1000) as u64;
        let duration = Duration::new(secs, 0); // Just round off the nanos
        Timestamp::new(UNIX_EPOCH + duration).map_err(Into::into)
    }

    /// Lower the [`Timestamp`] to a [`js_sys::Date`]
    pub fn to_date(&self) -> js_sys::Date {
        js_sys::Date::new(&JsValue::from(
            self.time
                .duration_since(UNIX_EPOCH)
                .expect("time should be in range since it's getting a JS Date")
                .as_millis(),
        ))
    }
}

impl TryFrom<SystemTime> for Timestamp {
    type Error = OutOfRangeError;

    fn try_from(sys_time: SystemTime) -> Result<Timestamp, Self::Error> {
        Timestamp::new(sys_time)
    }
}

impl From<Timestamp> for SystemTime {
    fn from(js_time: Timestamp) -> Self {
        js_time.time
    }
}

impl From<Timestamp> for Ipld {
    fn from(timestamp: Timestamp) -> Self {
        timestamp.to_unix().into()
    }
}

impl TryFrom<Ipld> for Timestamp {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            // FIXME do bounds checking
            Ipld::Integer(secs) => Ok(Timestamp::new(
                UNIX_EPOCH + Duration::from_secs(secs as u64),
            )
            .map_err(|_| ())?),
            _ => Err(()),
        }
    }
}

impl From<Timestamp> for i128 {
    fn from(timestamp: Timestamp) -> i128 {
        timestamp.to_unix() as i128
    }
}

impl TryFrom<i128> for Timestamp {
    type Error = OutOfRangeError;

    fn try_from(secs: i128) -> Result<Self, Self::Error> {
        // FIXME do bounds checking
        Timestamp::new(UNIX_EPOCH + Duration::from_secs(secs as u64))
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_unix().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = u64::deserialize(deserializer)?;
        Ok(Timestamp::postel(UNIX_EPOCH + Duration::from_secs(seconds)))
    }
}

#[cfg(feature = "test_utils")]
impl Arbitrary for Timestamp {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0..(u64::pow(2, 53) - 1))
            .prop_map(|secs| {
                Timestamp::new(UNIX_EPOCH + Duration::from_secs(secs))
                    .expect("the current time to be somtime in the 3rd millenium CE")
            })
            .boxed()
    }
}
