use super::responds::Responds;
use crate::{capsule::Capsule, did::Did, nonce::Nonce, time::Timestamp};
use libipld_core::{cid::Cid, ipld::Ipld, serde as ipld_serde};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Payload<T: Responds>
where
    T::Success: Serialize + DeserializeOwned,
{
    pub issuer: Did,

    pub ran: Cid,
    pub out: Result<T::Success, BTreeMap<String, Ipld>>,
    pub next: Vec<Cid>,

    pub proofs: Vec<Cid>,
    pub metadata: BTreeMap<String, Ipld>,

    pub nonce: Nonce,
    pub issued_at: Option<Timestamp>,
}

impl<T: Responds> Capsule for Payload<T>
where
    for<'de> T::Success: Serialize + Deserialize<'de>,
{
    const TAG: &'static str = "ucan/r/1.0.0-rc.1"; // FIXME extract out version
}

impl<T: Responds> TryFrom<Ipld> for Payload<T>
where
    for<'de> T::Success: Serialize + Deserialize<'de>,
{
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

impl<T: Responds> From<Payload<T>> for Ipld
where
    for<'de> T::Success: Serialize + Deserialize<'de>,
{
    fn from(payload: Payload<T>) -> Self {
        payload.into()
    }
}
