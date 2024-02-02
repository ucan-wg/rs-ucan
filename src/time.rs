//! Time utilities

use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use web_time::{Duration, SystemTime, UNIX_EPOCH};

/// Get the current time in seconds since UNIX_EPOCH
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Debug, Clone, PartialEq)]
pub enum Timestamp {
    // FIXME probably overkill, but overflows are bad. Need to check on ingestion, too
    // Per the spec, timestamps MUST respect [IEEE-754](https://en.wikipedia.org/wiki/IEEE_754)
    // (64-bit double precision = 53-bit truncated integer) for JavaScript interoperability.
    //
    // This range can represent millions of years into the future,
    // and is thus sufficient for nearly all use cases.
    Sending(JsTime),

    /// Following [Postel's Law](https://en.wikipedia.org/wiki/Robustness_principle),
    /// received timestamps may be parsed as regular [SystemTime]
    Receiving(SystemTime),
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Timestamp::Sending(js_time) => js_time.serialize(serializer),
            Timestamp::Receiving(sys_time) => todo!(), // FIXME See comment on deserilaizer sys_time.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Ok(js_time) = JsTime::deserialize(deserializer) {
            return Ok(Timestamp::Sending(js_time));
        }

        todo!()
        // FIXME just todo()ing this for now becuase the enum will likely go away very shortly
    }
}

impl From<Timestamp> for SystemTime {
    fn from(timestamp: Timestamp) -> Self {
        match timestamp {
            Timestamp::Sending(js_time) => js_time.time,
            Timestamp::Receiving(sys_time) => sys_time,
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsTime {
    time: SystemTime,
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
