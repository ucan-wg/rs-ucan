use super::resolvable::Resolvable;
use crate::{
    ability::{arguments, command::ToCommand},
    capsule::Capsule,
    delegation::{self, condition::Condition, error::DelegationError, Delegable},
    did::{self, Did},
    nonce::Nonce,
    proof::{checkable::Checkable, prove::Prove},
    time::Timestamp,
};
use libipld_core::{cid::Cid, error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Serialize, Serializer};
use std::{collections::BTreeMap, fmt::Debug};
use web_time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<T, DID: Did> {
    pub issuer: DID,
    pub subject: DID,
    pub audience: Option<DID>,

    pub ability: T,

    pub proofs: Vec<Cid>,
    pub cause: Option<Cid>,
    pub metadata: BTreeMap<String, Ipld>,
    pub nonce: Nonce,

    pub not_before: Option<Timestamp>,
    pub expiration: Timestamp,
}

// FIXME cleanup traits
// one idea, because they keep comingup together: put hierarchy and builder on the same
// trair (as associated tyeps) to klet us skip the ::bulder::hierarchy indirection.
//
// This probably means putting the delegation T back to the upper level and bieng explicit about
// the T::Builder in the type
impl<T, DID: Did + Clone> Payload<T, DID> {
    pub fn check<C: Condition>(
        self,
        proofs: Vec<delegation::Payload<<T::Builder as Checkable>::Hierarchy, C, DID>>,
        now: SystemTime,
    ) -> Result<(), DelegationError<<<T::Builder as Checkable>::Hierarchy as Prove>::Error>>
    where
        T: Delegable,
        T::Builder: Clone + Checkable + Prove + Into<arguments::Named<Ipld>>,
        <T::Builder as Checkable>::Hierarchy: Clone + Into<arguments::Named<Ipld>>,
    {
        let builder_payload: delegation::Payload<T::Builder, C, DID> = self.into();
        builder_payload.check(proofs, now)
    }
}

impl<T, DID: Did> Capsule for Payload<T, DID> {
    const TAG: &'static str = "ucan/i/1.0.0-rc.1";
}

impl<T: Delegable, C: Condition, DID: Did + Clone> From<Payload<T, DID>>
    for delegation::Payload<T::Builder, C, DID>
{
    fn from(payload: Payload<T, DID>) -> Self {
        delegation::Payload {
            issuer: payload.issuer.clone(),
            subject: payload.subject.clone(),
            audience: payload.audience.unwrap_or(payload.subject),

            ability_builder: T::Builder::from(payload.ability),
            conditions: vec![],

            metadata: payload.metadata,
            nonce: payload.nonce,

            not_before: payload.not_before,
            expiration: payload.expiration,
        }
    }
}

impl<T: ToCommand + Into<Ipld>, DID: Did> From<Payload<T, DID>> for arguments::Named<Ipld> {
    fn from(payload: Payload<T, DID>) -> Self {
        let mut args = arguments::Named::from_iter([
            ("iss".into(), payload.issuer.into().to_string().into()),
            ("sub".into(), payload.subject.into().to_string().into()),
            ("cmd".into(), payload.ability.to_command().into()),
            ("args".into(), payload.ability.into()),
            (
                "prf".into(),
                Ipld::List(payload.proofs.iter().map(Into::into).collect()),
            ),
            ("nonce".into(), payload.nonce.into()),
            ("exp".into(), payload.expiration.into()),
        ]);

        if let Some(audience) = payload.audience {
            args.insert("aud".into(), audience.into().to_string().into());
        }

        if let Some(not_before) = payload.not_before {
            args.insert("nbf".into(), not_before.into());
        }

        args
    }
}

/// A variant that accepts [`Promise`]s.
///
/// [`Promise`]: crate::invocation::promise::Promise
pub type Promised<T, DID> = Payload<<T as Resolvable>::Promised, DID>;

impl<T: Resolvable, DID: Did> Resolvable for Payload<T, DID>
where
    arguments::Named<Ipld>: From<T::Promised>,
    Ipld: From<T::Promised>,
    T::Promised: ToCommand,
{
    type Promised = Promised<T, DID>;

    fn try_resolve(promised: Promised<T, DID>) -> Result<Self, Self::Promised> {
        match <T as Resolvable>::try_resolve(promised.ability) {
            Ok(resolved_ability) => Ok(Payload {
                issuer: promised.issuer,
                subject: promised.subject,
                audience: promised.audience,

                ability: resolved_ability,

                proofs: promised.proofs,
                cause: promised.cause,
                metadata: promised.metadata,
                nonce: promised.nonce,

                not_before: promised.not_before,
                expiration: promised.expiration,
            }),
            Err(promised_ability) => Err(Payload {
                ability: promised_ability,
                ..promised
            }),
        }
    }
}

impl<T, DID: Did> Serialize for Payload<T, DID>
where
    Payload<T, DID>: Clone,
    InternalSerializer: From<Payload<T, DID>>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = InternalSerializer::from(self.clone());
        serde::Serialize::serialize(&s, serializer)
    }
}

impl<'de, T, DID: Did> serde::Deserialize<'de> for Payload<T, DID>
where
    Payload<T, DID>: TryFrom<InternalSerializer>,
    <Payload<T, DID> as TryFrom<InternalSerializer>>::Error: Debug,
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

impl<T, DID: Did> TryFrom<Ipld> for Payload<T, DID>
where
    Payload<T, DID>: TryFrom<InternalSerializer>,
{
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let s: InternalSerializer = ipld_serde::from_ipld(ipld).map_err(|_| ())?;
        s.try_into().map_err(|_| ()) // FIXME
    }
}

impl<T, DID: Did> From<Payload<T, DID>> for Ipld {
    fn from(payload: Payload<T, DID>) -> Self {
        payload.into()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct InternalSerializer {
    #[serde(rename = "iss")]
    issuer: did::Newtype,
    #[serde(rename = "sub")]
    subject: did::Newtype,
    #[serde(rename = "aud", skip_serializing_if = "Option::is_none")]
    audience: Option<did::Newtype>,

    #[serde(rename = "cmd")]
    command: String,
    #[serde(rename = "args")]
    arguments: arguments::Named<Ipld>,

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

impl<T: ToCommand + Into<arguments::Named<Ipld>>, DID: Did> From<Payload<T, DID>>
    for InternalSerializer
{
    fn from(payload: Payload<T, DID>) -> Self {
        InternalSerializer {
            issuer: payload.issuer.into(),
            subject: payload.subject.into(),
            audience: payload.audience.map(Into::into),

            command: payload.ability.to_command(),
            arguments: payload.ability.into(),

            proofs: payload.proofs,
            cause: payload.cause,
            metadata: payload.metadata,

            nonce: payload.nonce,

            not_before: payload.not_before,
            expiration: payload.expiration,
        }
    }
}
