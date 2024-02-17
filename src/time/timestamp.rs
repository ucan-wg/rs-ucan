use super::JsTime;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use web_time::{Duration, SystemTime, UNIX_EPOCH};

/// All timestamps that this library can handle.
///
/// Strictly speaking, UCAN exclusively supports [`JsTime`] (for JavaScript interoperability).
/// While this library only allows creation of [`JsTime`]s, it will parse the broader
/// [`SystemTime`] range to be liberal in what it accepts. Large numbers are only a problem in
/// langauges that lack 64-bit integers (like JavaScript).
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Timestamp {
    /// An entry for [`JsTime`], which is compatible with JavaScript's 2⁵³ numeric range.
    JsSafe(JsTime),

    /// Following [Postel's Law](https://en.wikipedia.org/wiki/Robustness_principle),
    /// received timestamps may be parsed as regular [`SystemTime`].
    Postel(SystemTime),
}

impl Timestamp {
    /// Get the current time in seconds since [`UNIX_EPOCH`] as a [`Timestamp`].
    ///
    /// This will always return the [`JsSafe`][Timestamp::JsSafe] variant.
    pub fn now() -> Timestamp {
        Timestamp::JsSafe(JsTime::now())
    }
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
            Timestamp::Postel(sys_time) => sys_time.serialize(serializer),
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
