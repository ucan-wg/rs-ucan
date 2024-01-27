//! Time utilities

use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};
use web_time::{SystemTime, UNIX_EPOCH};

/// Get the current time in seconds since UNIX_EPOCH
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Timestamp {
    // FIXME probably overkill, but overflows are bad. Need to check on ingestion, too
    /// Per the spec, timestamps MUST respect [IEEE-754](https://en.wikipedia.org/wiki/IEEE_754)
    /// (64-bit double precision = 53-bit truncated integer) for JavaScript interoperability.
    ///
    /// This range can represent millions of years into the future,
    /// and is thus sufficient for nearly all use cases.
    Sending(JsTime),

    /// Following [Postel's Law](https://en.wikipedia.org/wiki/Robustness_principle),
    /// received timestamps may be parsed as regular [SystemTime]
    Receiving(SystemTime),
}

impl From<Timestamp> for Ipld {
    fn from(timestamp: Timestamp) -> Self {
        timestamp.into()
    }
}

impl TryFrom<Ipld> for Timestamp {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsTime {
    time: SystemTime,
}

// FIXME just lifting this from Elixir for now
pub struct OutOfRangeError {
    pub tried: SystemTime,
}

impl JsTime {
    /// Create a [`JsTime`] from a [`SystemTime`]
    ///
    /// # Errors
    ///
    /// * [`OutOfRangeError`] — If the time is more than 2⁵³ seconds since the Unix epoch
    pub fn new(time: SystemTime) -> Result<Self, OutOfRangeError> {
        if time
            .duration_since(std::time::UNIX_EPOCH)
            .expect("FIXME")
            .as_secs()
            > 0x1FFFFFFFFFFFFF
        {
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
            .duration_since(std::time::UNIX_EPOCH)
            .expect("FIXME")
            .as_secs()
            .into()
    }
}
