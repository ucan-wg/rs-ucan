use crate::{
    ability::traits::{Command, DynJs, Resolvable},
    capsule::Capsule,
    did::Did,
    nonce::Nonce,
    time::Timestamp,
};
use libipld_core::{cid::Cid, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize, Serializer};
use std::{collections::BTreeMap, fmt::Debug};

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<T: Resolvable> {
    pub issuer: Did,
    pub subject: Did,
    pub audience: Option<Did>,

    pub ability: T::Awaiting,

    pub proofs: Vec<Cid>,
    pub cause: Option<Cid>,
    pub metadata: BTreeMap<String, Ipld>, // FIXME parameterize?
    pub nonce: Nonce,

    pub not_before: Option<Timestamp>,
    pub expiration: Timestamp,
}

impl<T: Resolvable + Command> Capsule for Payload<T> {
    const TAG: &'static str = "ucan/i/1.0.0-rc.1";
}

impl<T: Resolvable> Serialize for Payload<T>
where
    Payload<T>: Clone,
    InternalSerializer: From<Payload<T>>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = InternalSerializer::from(self.clone());
        Serialize::serialize(&s, serializer)
    }
}

impl<'de, T: Resolvable> Deserialize<'de> for Payload<T>
where
    Payload<T>: TryFrom<InternalSerializer>,
    <Payload<T> as TryFrom<InternalSerializer>>::Error: Debug,
{
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match InternalSerializer::deserialize(d) {
            Err(e) => Err(e),
            Ok(s) => s
                .try_into()
                .map_err(|e| serde::de::Error::custom(format!("{:?}", e))), // FIXME better error
        }
    }
}

impl<T: Resolvable> TryFrom<Ipld> for Payload<T>
where
    Payload<T>: TryFrom<InternalSerializer>,
{
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let s: InternalSerializer = ipld_serde::from_ipld(ipld).map_err(|_| ())?;
        s.try_into().map_err(|_| ()) // FIXME
    }
}

impl<T: Resolvable> From<Payload<T>> for Ipld {
    fn from(payload: Payload<T>) -> Self {
        payload.into()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InternalSerializer {
    #[serde(rename = "iss")]
    issuer: Did,
    #[serde(rename = "sub")]
    subject: Did,
    #[serde(rename = "aud", skip_serializing_if = "Option::is_none")]
    audience: Option<Did>,

    #[serde(rename = "do")]
    command: String,
    #[serde(rename = "args")]
    arguments: BTreeMap<String, Ipld>,

    #[serde(rename = "prf")]
    proofs: Vec<Cid>,
    #[serde(rename = "nonce")]
    nonce: Nonce,

    #[serde(rename = "cause")]
    cause: Option<Cid>,
    #[serde(rename = "meta")]
    metadata: BTreeMap<String, Ipld>,

    #[serde(rename = "nbf", skip_serializing_if = "Option::is_none")]
    not_before: Option<Timestamp>,
    #[serde(rename = "exp")]
    expiration: Timestamp,
}

impl<T: Resolvable + Command + Debug> From<Payload<T>> for InternalSerializer
where
    BTreeMap<String, Ipld>: From<T::Awaiting>,
    Ipld: From<T::Awaiting>,
{
    fn from(payload: Payload<T>) -> Self {
        let arguments: BTreeMap<String, Ipld> = match Ipld::from(payload.ability) {
            Ipld::Map(btree) => btree,
            _ => panic!("FIXME"),
        };

        InternalSerializer {
            issuer: payload.issuer,
            subject: payload.subject,
            audience: payload.audience,

            command: T::COMMAND.into(),
            arguments,

            proofs: payload.proofs,
            cause: payload.cause,
            metadata: payload.metadata,

            nonce: payload.nonce,

            not_before: payload.not_before,
            expiration: payload.expiration,
        }
    }
}

impl TryFrom<Ipld> for InternalSerializer {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, ()> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

impl From<InternalSerializer> for Payload<DynJs> {
    fn from(s: InternalSerializer) -> Self {
        Payload {
            issuer: s.issuer,
            subject: s.subject,
            audience: s.audience,

            ability: DynJs {
                cmd: s.command,
                args: s.arguments,
            },

            proofs: s.proofs,
            cause: s.cause,
            metadata: s.metadata,

            nonce: s.nonce,

            not_before: s.not_before,
            expiration: s.expiration,
        }
    }
}

impl TryFrom<Payload<DynJs>> for InternalSerializer {
    type Error = (); // FIXME

    fn try_from(p: Payload<DynJs>) -> Result<Self, ()> {
        Ok(InternalSerializer {
            issuer: p.issuer,
            subject: p.subject,
            audience: p.audience,

            command: p.ability.cmd,
            arguments: p.ability.args,

            proofs: p.proofs,
            cause: p.cause,
            metadata: p.metadata,

            nonce: p.nonce,

            not_before: p.not_before,
            expiration: p.expiration,
        })
    }
}
