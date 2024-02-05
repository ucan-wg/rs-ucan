use super::resolvable::Resolvable;
use crate::{
    ability::{arguments::Arguments, command::Command},
    capsule::Capsule,
    did::Did,
    metadata as meta,
    metadata::{Mergable, Metadata},
    nonce::Nonce,
    time::Timestamp,
};
use libipld_core::{cid::Cid, error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Serialize, Serializer};
use std::{collections::BTreeMap, fmt::Debug};

// FIXME this version should not be resolvable...
// FIXME ...or at least have two versions via abstraction
#[derive(Debug, Clone, PartialEq)]
pub struct Payload<T, E: meta::Entries> {
    pub issuer: Did,
    pub subject: Did,
    pub audience: Option<Did>,

    pub ability: T,

    pub proofs: Vec<Cid>,
    pub cause: Option<Cid>,
    pub metadata: Metadata<E>,
    pub nonce: Nonce,

    pub not_before: Option<Timestamp>,
    pub expiration: Timestamp,
}

// NOTE This is the version that accepts promises
pub type Unresolved<T: Resolvable, E: meta::Entries> = Payload<T::Promised, E>;
// type Dynamic = Payload<dynamic::Dynamic>; <- ?

// FIXME parser for both versions
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum MaybeResolved<T: Resolvable + Command + Clone + TryFrom<Arguments> + Into<Arguments>>
// where
//     Payload<T>: From<InternalSerializer>,
//     Unresolved<T>: From<InternalSerializer>,
//     T::Promised: Clone + Command + Debug + PartialEq,
// {
//     Resolved(Payload<T>),
//     Unresolved(Unresolved<T>),
// }

impl<T, E: meta::Entries> Capsule for Payload<T, E> {
    const TAG: &'static str = "ucan/i/1.0.0-rc.1";
}

impl<T, E: meta::Entries> Serialize for Payload<T, E>
where
    Payload<T, E>: Clone,
    InternalSerializer: From<Payload<T, E>>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = InternalSerializer::from(self.clone());
        serde::Serialize::serialize(&s, serializer)
    }
}

impl<'de, T, E: meta::Entries> serde::Deserialize<'de> for Payload<T, E>
where
    Payload<T, E>: TryFrom<InternalSerializer>,
    <Payload<T, E> as TryFrom<InternalSerializer>>::Error: Debug,
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

impl<T, E: meta::Entries> TryFrom<Ipld> for Payload<T, E>
where
    Payload<T, E>: TryFrom<InternalSerializer>,
{
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let s: InternalSerializer = ipld_serde::from_ipld(ipld).map_err(|_| ())?;
        s.try_into().map_err(|_| ()) // FIXME
    }
}

impl<T, E: meta::Entries> From<Payload<T, E>> for Ipld {
    fn from(payload: Payload<T, E>) -> Self {
        payload.into()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct InternalSerializer {
    #[serde(rename = "iss")]
    issuer: Did,
    #[serde(rename = "sub")]
    subject: Did,
    #[serde(rename = "aud", skip_serializing_if = "Option::is_none")]
    audience: Option<Did>,

    #[serde(rename = "cmd")]
    command: String,
    #[serde(rename = "args")]
    arguments: Arguments,

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

impl From<InternalSerializer> for Ipld {
    fn from(serializer: InternalSerializer) -> Self {
        serializer.into()
    }
}

impl TryFrom<Ipld> for InternalSerializer {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

// FIXME
// impl From<InternalSerializer> for Payload<dynamic::Dynamic> {
//     fn from(s: InternalSerializer) -> Self {
//         Payload {
//             issuer: s.issuer,
//             subject: s.subject,
//             audience: s.audience,
//
//             ability: dynamic::Dynamic {
//                 cmd: s.command,
//                 args: s.arguments.into(),
//             },
//
//             proofs: s.proofs,
//             cause: s.cause,
//             metadata: s.metadata,
//
//             nonce: s.nonce,
//
//             not_before: s.not_before,
//             expiration: s.expiration,
//         }
//     }
// }

// FIXME
// impl From<Payload<dynamic::Dynamic>> for InternalSerializer {
//     fn from(p: Payload<dynamic::Dynamic>) -> Self {
//         InternalSerializer {
//             issuer: p.issuer,
//             subject: p.subject,
//             audience: p.audience,
//
//             command: p.ability.cmd,
//             arguments: p.ability.args,
//
//             proofs: p.proofs,
//             cause: p.cause,
//             metadata: p.metadata,
//
//             nonce: p.nonce,
//
//             not_before: p.not_before,
//             expiration: p.expiration,
//         }
//     }
// }

impl<T: Command + Into<Arguments>, E: meta::Entries> From<Payload<T, E>> for InternalSerializer {
    fn from(payload: Payload<T, E>) -> Self {
        InternalSerializer {
            issuer: payload.issuer,
            subject: payload.subject,
            audience: payload.audience,

            command: T::COMMAND.into(),
            arguments: payload.ability.into(),

            proofs: payload.proofs,
            cause: payload.cause,
            metadata: payload.metadata.merge(),

            nonce: payload.nonce,

            not_before: payload.not_before,
            expiration: payload.expiration,
        }
    }
}
