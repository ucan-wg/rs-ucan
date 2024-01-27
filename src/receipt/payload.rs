use crate::{ability::traits::Runnable, capsule::Capsule, did::Did, nonce::Nonce, time::Timestamp};
use libipld_core::{cid::Cid, ipld::Ipld, serde as ipld_serde};
use serde::{de::DeserializeOwned, Deserialize, Serialize, Serializer};
use std::{collections::BTreeMap, fmt::Debug};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Payload<T: Runnable>
where
    T::Output: Serialize + DeserializeOwned,
{
    pub issuer: Did,

    pub ran: Cid,
    pub out: Result<T::Output, BTreeMap<String, Ipld>>,
    pub next: Vec<Cid>,

    pub proofs: Vec<Cid>,
    pub metadata: BTreeMap<String, Ipld>,

    pub nonce: Nonce,
    pub issued_at: Option<Timestamp>,
}

impl<T: Runnable> Capsule for Payload<T>
where
    for<'de> T::Output: Serialize + Deserialize<'de>,
{
    const TAG: &'static str = "ucan/r/1.0.0-rc.1"; // FIXME extract out version
}

impl<T: Runnable> TryFrom<Ipld> for Payload<T>
where
    for<'de> T::Output: Serialize + Deserialize<'de>,
{
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

impl<T: Runnable> From<Payload<T>> for Ipld
where
    for<'de> T::Output: Serialize + Deserialize<'de>,
{
    fn from(payload: Payload<T>) -> Self {
        payload.into()
    }
}
