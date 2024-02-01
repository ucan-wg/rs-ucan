use super::responds::Responds;
use crate::{
    ability::arguments::Arguments,
    capsule::Capsule,
    did::Did,
    metadata as meta,
    metadata::{Mergable, Metadata},
    nonce::Nonce,
    time::Timestamp,
};
use libipld_core::{cid::Cid, ipld::Ipld, serde as ipld_serde};
use serde::{de::DeserializeOwned, Deserialize, Serialize, Serializer};
use std::{collections::BTreeMap, fmt::Debug};

// FIXME serialize/deseialize split out for when the T has implementations

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<T: Responds, E: meta::Entries> {
    pub issuer: Did,

    pub ran: Cid,
    pub out: Result<T::Success, Arguments>,
    pub next: Vec<Cid>, // FIXME rename here or in spec?

    pub proofs: Vec<Cid>,
    pub metadata: Metadata<E>,

    pub nonce: Nonce,
    pub issued_at: Option<Timestamp>,
}

impl<T: Responds, E: meta::Entries> Capsule for Payload<T, E> {
    const TAG: &'static str = "ucan/r/1.0.0-rc.1"; // FIXME extract out version
}

impl<T: Responds + Serialize, E: meta::Entries + Clone> Serialize for Payload<T, E>
where
    Payload<T, E>: Clone,
    T::Success: Serialize + DeserializeOwned,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = InternalSerializer::from(self.clone()); // FIXME kill that clone with tons of refs?
        serde::Serialize::serialize(&s, serializer)
    }
}

impl<
        'de,
        T: Responds + Deserialize<'de>,
        E: meta::Entries + Clone + DeserializeOwned + Serialize,
    > Deserialize<'de> for Payload<T, E>
where
    <Payload<T, E> as TryFrom<InternalSerializer<T>>>::Error: Debug,
    T::Success: DeserializeOwned + Serialize,
{
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match InternalSerializer::deserialize(d) {
            Err(e) => Err(e),
            Ok(s) => Ok(s.into()),
        }
    }
}

impl<T: Responds, E: meta::Entries + Serialize + DeserializeOwned + Clone> TryFrom<Ipld>
    for Payload<T, E>
where
    T::Success: Serialize + DeserializeOwned,
{
    type Error = (); // FIXME serde error

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let s: InternalSerializer<T> = ipld_serde::from_ipld(ipld).map_err(|_| ())?;
        Ok(s.into())
    }
}

impl<T: Responds, E: meta::Entries> From<Payload<T, E>> for Ipld {
    fn from(payload: Payload<T, E>) -> Self {
        payload.into()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InternalSerializer<T: Responds>
where
    T::Success: Serialize + DeserializeOwned,
{
    #[serde(rename = "iss")]
    issuer: Did,

    ran: Cid,
    out: Result<T::Success, Arguments>,
    next: Vec<Cid>, // FIXME rename here or in spec?

    #[serde(rename = "prf")]
    proofs: Vec<Cid>,
    #[serde(rename = "meta")]
    metadata: BTreeMap<String, Ipld>,

    nonce: Nonce,
    #[serde(rename = "iat")]
    issued_at: Option<Timestamp>,
}

impl<T: Responds, E: meta::Entries + Clone + Serialize> From<InternalSerializer<T>>
    for Payload<T, E>
where
    T::Success: Serialize + DeserializeOwned,
    Metadata<E>: Mergable,
{
    fn from(s: InternalSerializer<T>) -> Self {
        Payload {
            issuer: s.issuer,
            ran: s.ran,
            out: s.out,
            next: s.next,
            proofs: s.proofs,
            metadata: Metadata::extract(s.metadata),
            nonce: s.nonce,
            issued_at: s.issued_at,
        }
    }
}

impl<T: Responds, E: meta::Entries + Clone> From<Payload<T, E>> for InternalSerializer<T>
where
    T::Success: Serialize + DeserializeOwned,
    Metadata<E>: Mergable,
{
    fn from(s: Payload<T, E>) -> Self {
        InternalSerializer {
            issuer: s.issuer,
            ran: s.ran,
            out: s.out,
            next: s.next,
            proofs: s.proofs,
            metadata: s.metadata.merge(),
            nonce: s.nonce,
            issued_at: s.issued_at,
        }
    }
}
