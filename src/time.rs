//! Time utilities.

use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use thiserror::Error;
use web_time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Get the current time in seconds since [`UNIX_EPOCH`].
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// All timestamps that this library can handle.
///
/// Strictly speaking, UCAN exclusively supports [`JsTime`] (for JavaScript interoperability).
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Timestamp {
    /// An entry for [`JsTime`], which is compatible with JavaScript's 2⁵³ numeric range.
    JsSafe(JsTime),

    /// Following [Postel's Law](https://en.wikipedia.org/wiki/Robustness_principle),
    /// received timestamps may be parsed as regular [`SystemTime`].
    Postel(SystemTime),
}

impl From<JsTime> for Timestamp {
    fn from(js_time: JsTime) -> Self {
        Timestamp::JsSafe(js_time)
    }
}

impl From<SystemTime> for Timestamp {
    fn from(sys_time: SystemTime) -> Self {
        Timestamp::Postel(sys_time)
    }
}

impl From<Timestamp> for Ipld {
    fn from(timestamp: Timestamp) -> Self {
        match timestamp {
            Timestamp::JsSafe(js_time) => js_time.into(),
            Timestamp::Postel(sys_time) => sys_time
                .duration_since(UNIX_EPOCH)
                .expect("FIXME")
                .as_secs()
                .into(),
        }
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Timestamp::JsSafe(js_time) => js_time.serialize(serializer),
            Timestamp::Postel(_sys_time) => todo!(), // FIXME See comment on deserilaizer sys_time.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Ok(secs) = u64::deserialize(deserializer) {
            match UNIX_EPOCH.checked_add(Duration::new(secs, 0)) {
                None => return Err(serde::de::Error::custom("time out of range for SystemTime")),
                Some(sys_time) => match JsTime::new(sys_time) {
                    Ok(js_time) => Ok(Timestamp::JsSafe(js_time)),
                    Err(_) => Ok(Timestamp::Postel(sys_time)),
                },
            }
        } else {
            Err(serde::de::Error::custom("not a Timestamp"))
        }
    }
}

impl From<Timestamp> for SystemTime {
    fn from(timestamp: Timestamp) -> Self {
        match timestamp {
            Timestamp::JsSafe(js_time) => js_time.time,
            Timestamp::Postel(sys_time) => sys_time,
        }
    }
}

impl TryFrom<Ipld> for Timestamp {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

/// A JavaScript-wrapper for [`Timestamp`].
///
/// Per the UCAN spec, timestamps MUST respect [IEEE-754]
/// (64-bit double precision = 53-bit truncated integer) for JavaScript interoperability.
///
/// This range can represent millions of years into the future,
/// and is thus sufficient for "nearly" all auth use cases.
///
/// [IEEE-754]: https://en.wikipedia.org/wiki/IEEE_754
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct JsTime {
    time: SystemTime,
}

impl From<JsTime> for SystemTime {
    fn from(js_time: JsTime) -> Self {
        js_time.time
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl JsTime {
    /// Lift a [`js_sys::Date`] into a Rust [`JsTime`]
    pub fn from_date(date_time: js_sys::Date) -> Result<JsTime, JsError> {
        let millis = date_time.get_time() as u64;
        let secs: u64 = (millis / 1000) as u64;
        let duration = Duration::new(secs, 0); // Just round off the nanos
        JsTime::new(UNIX_EPOCH + duration).map_err(Into::into)
    }

    /// Lower the [`JsTime`] to a [`js_sys::Date`]
    pub fn to_date(&self) -> js_sys::Date {
        js_sys::Date::new(&JsValue::from(
            self.time
                .duration_since(UNIX_EPOCH)
                .expect("time should be in range since it's getting a JS Date")
                .as_millis(),
        ))
    }
}

impl JsTime {
    /// Create a [`JsTime`] from a [`SystemTime`].
    ///
    /// # Arguments
    ///
    /// * `time` — The time to convert
    ///
    /// # Errors
    ///
    /// * [`OutOfRangeError`] — If the time is more than 2⁵³ seconds since the Unix epoch
    pub fn new(time: SystemTime) -> Result<Self, OutOfRangeError> {
        if time.duration_since(UNIX_EPOCH).expect("FIXME").as_secs() > 0x1FFFFFFFFFFFFF {
            Err(OutOfRangeError { tried: time })
        } else {
            Ok(JsTime { time })
        }
    }
}

impl From<JsTime> for Ipld {
    fn from(js_time: JsTime) -> Self {
        js_time
            .time
            .duration_since(UNIX_EPOCH)
            .expect("FIXME")
            .as_secs()
            .into()
    }
}

impl Serialize for JsTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.time
            .duration_since(UNIX_EPOCH)
            .expect("FIXME")
            .as_secs()
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for JsTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = u64::deserialize(deserializer)?;
        JsTime::new(UNIX_EPOCH + Duration::from_secs(seconds))
            .map_err(|_| serde::de::Error::custom("time out of JsTime range"))
    }
}

/// An error expressing when a time is larger than 2⁵³ seconds past the Unix epoch
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub struct OutOfRangeError {
    /// The [`SystemTime`] that is outside of the [`JsTime`] range (2⁵³).
    pub tried: SystemTime,
}

impl fmt::Display for OutOfRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "time out of JsTime (2⁵³) range: {:?}", self.tried)
    }
}

// FIXME move to time.rs
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Error)]
pub enum TimeBoundError {
    #[error("The UCAN delegation has expired")]
    Expired,

    #[error("The UCAN delegation is not yet valid")]
    NotYetValid,
}
