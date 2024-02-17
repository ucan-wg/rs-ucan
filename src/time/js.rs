//! A JavaScript-wrapper for [`Timestamp`][crate::time::Timestamp].

use super::OutOfRangeError;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use web_time::{Duration, SystemTime, UNIX_EPOCH};

/// A JavaScript-wrapper for [`Timestamp`][super::Timestamp].
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
    pub time: SystemTime,
}

impl JsTime {
    /// Get the current time in seconds since [`UNIX_EPOCH`] as a [`JsTime`].
    pub fn now() -> JsTime {
        Self::new(SystemTime::now())
            .expect("the current time to be somtime in the 3rd millenium CE")
    }

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

impl From<JsTime> for SystemTime {
    fn from(js_time: JsTime) -> Self {
        js_time.time
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
