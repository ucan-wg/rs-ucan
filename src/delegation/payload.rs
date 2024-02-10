use super::{
    condition::Condition,
    error::{DelegationError, EnvelopeError},
};
use crate::{
    ability::{arguments, command::Command},
    capsule::Capsule,
    did::Did,
    nonce::Nonce,
    proof::{
        checkable::Checkable,
        prove::{Prove, Success},
        same::CheckSame,
    },
    time::Timestamp,
};
use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde::{de::DeserializeOwned, Deserialize, Serialize, Serializer};
use std::{collections::BTreeMap, fmt::Debug};
use web_time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<D, C: Condition> {
    pub issuer: Did,
    pub subject: Did,
    pub audience: Did,

    pub delegated_ability: D,
    pub conditions: Vec<C>, // FIXME BTreeSet?
    pub metadata: BTreeMap<String, Ipld>,
    pub nonce: Nonce,

    pub expiration: Timestamp,
    pub not_before: Option<Timestamp>,
}

impl<D, C: Condition> Capsule for Payload<D, C> {
    const TAG: &'static str = "ucan/d/1.0.0-rc.1";
}

impl<D, C: Condition + Serialize> Serialize for Payload<D, C>
where
    InternalSerializer: From<Payload<D, C>>,
    Payload<D, C>: Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = InternalSerializer::from(self.clone()); // FIXME
        Serialize::serialize(&s, serializer)
    }
}

impl<'de, T, C: Condition + DeserializeOwned> Deserialize<'de> for Payload<T, C>
where
    Payload<T, C>: TryFrom<InternalSerializer>,
    <Payload<T, C> as TryFrom<InternalSerializer>>::Error: Debug,
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

impl<T, C: Condition + Serialize + DeserializeOwned> TryFrom<Ipld> for Payload<T, C>
where
    Payload<T, C>: TryFrom<InternalSerializer>,
{
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let s: InternalSerializer = ipld_serde::from_ipld(ipld).map_err(|_| ())?;
        s.try_into().map_err(|_| ()) // FIXME
    }
}

impl<T, C: Condition> From<Payload<T, C>> for Ipld {
    fn from(payload: Payload<T, C>) -> Self {
        payload.into()
    }
}

// FIXME this likely should move to invocation
impl<'a, T: Checkable + CheckSame + Clone + Prove + Into<arguments::Named<Ipld>>, C: Condition>
    Payload<T, C>
{
    pub fn check(
        delegated: &'a Payload<T, C>, // FIXME promisory version
        proofs: Vec<Payload<T, C>>,
        now: SystemTime,
    ) -> Result<(), DelegationError<<T as Prove>::Error>>
where {
        let start: Acc<T> = Acc {
            issuer: delegated.issuer.clone(),
            subject: delegated.subject.clone(),
            hierarchy: delegated.delegated_ability.clone(),
        };

        let args: arguments::Named<Ipld> = delegated.delegated_ability.clone().into();

        proofs
            .into_iter()
            .fold(Ok(start), |prev, proof| {
                if let Ok(prev_) = prev {
                    step(&prev_, &proof, &args, now).map(move |success| {
                        match success {
                            Success::ProvenByAny => Acc {
                                issuer: proof.issuer.clone(),
                                subject: proof.subject.clone(),
                                hierarchy: prev_.hierarchy,
                            },
                            Success::Proven => Acc {
                                issuer: proof.issuer.clone(),
                                subject: proof.subject.clone(),
                                hierarchy: proof.delegated_ability.clone(), // FIXME double check
                            },
                        }
                    })
                } else {
                    prev
                }
            })
            .map(|_| ())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Acc<T: Prove> {
    issuer: Did,
    subject: Did,
    hierarchy: T,
}

// FIXME this should move to Delegatable?
pub(crate) fn step<'a, T: Prove + Clone + Into<arguments::Named<Ipld>>, C: Condition>(
    prev: &Acc<T>,
    proof: &Payload<T, C>,
    args: &arguments::Named<Ipld>,
    now: SystemTime,
) -> Result<Success, DelegationError<<T as Prove>::Error>> {
    if let Err(_) = prev.issuer.check_same(&proof.audience) {
        return Err(EnvelopeError::InvalidSubject.into());
    }

    if let Err(_) = prev.subject.check_same(&proof.subject) {
        return Err(EnvelopeError::MisalignedIssAud.into());
    }

    if SystemTime::from(proof.expiration.clone()) > now {
        return Err(EnvelopeError::Expired.into());
    }

    if let Some(nbf) = proof.not_before.clone() {
        if SystemTime::from(nbf) > now {
            return Err(EnvelopeError::NotYetValid.into());
        }
    }

    // FIXME coudl be more efficient, but sets need Ord and we have floats
    for c in proof.conditions.iter() {
        // FIXME revisit
        if !c.validate(&args) || !c.validate(&prev.hierarchy.clone().into()) {
            return Err(DelegationError::FailedCondition);
        }
    }

    prev.hierarchy
        .check(&proof.delegated_ability.clone())
        .map_err(DelegationError::SemanticError)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InternalSerializer {
    #[serde(rename = "iss")]
    issuer: Did,
    #[serde(rename = "sub")]
    subject: Did,
    #[serde(rename = "aud")]
    audience: Did,

    #[serde(rename = "cmd")]
    command: String,
    #[serde(rename = "args")]
    arguments: arguments::Named<Ipld>,
    #[serde(rename = "cond")]
    conditions: Vec<Ipld>,

    #[serde(rename = "nonce")]
    nonce: Nonce,
    #[serde(rename = "meta")]
    metadata: BTreeMap<String, Ipld>,

    #[serde(rename = "nbf", skip_serializing_if = "Option::is_none")]
    not_before: Option<Timestamp>,
    #[serde(rename = "exp")]
    expiration: Timestamp,
}

impl<B: Command, C: Condition + Into<Ipld>> From<Payload<B, C>> for InternalSerializer
where
    arguments::Named<Ipld>: From<B>,
{
    fn from(payload: Payload<B, C>) -> Self {
        InternalSerializer {
            issuer: payload.issuer,
            subject: payload.subject,
            audience: payload.audience,

            command: B::COMMAND.into(),
            arguments: payload.delegated_ability.into(),
            conditions: payload.conditions.into_iter().map(|c| c.into()).collect(),

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

// FIXME
// impl<C: Condition + TryFrom<Ipld>, E: meta::MultiKeyed + Clone> TryFrom<InternalSerializer>
//     for Payload<dynamic::Dynamic<F>, C, E>
// {
//     type Error = (); // FIXME
//
//     fn try_from(s: InternalSerializer) -> Result<Payload<dynamic::Dynamic<F>, C, E>, ()> {
//         Ok(Payload {
//             issuer: s.issuer,
//             subject: s.subject,
//             audience: s.audience,
//
//             ability_builder: dynamic::Dynamic {
//                 cmd: s.command,
//                 args: s.arguments,
//             },
//             conditions: s
//                 .conditions
//                 .iter()
//                 .try_fold(Vec::new(), |mut acc, c| {
//                     C::try_from(c.clone()).map(|x| {
//                         acc.push(x);
//                         acc
//                     })
//                 })
//                 .map_err(|_| ())?, // FIXME better error (collect all errors
//
//             metadata: Metadata::extract(s.metadata),
//             nonce: s.nonce,
//
//             not_before: s.not_before,
//             expiration: s.expiration,
//         })
//     }
// }
//
// impl<C: Condition + Into<Ipld>, E: meta::MultiKeyed + Clone, F>
//     From<Payload<dynamic::Dynamic<F>, C, E>> for InternalSerializer
// where
//     Metadata<E>: Mergable,
// {
//     fn from(p: Payload<dynamic::Dynamic<F>, C, E>) -> Self {
//         InternalSerializer {
//             issuer: p.issuer,
//             subject: p.subject,
//             audience: p.audience,
//
//             command: p.ability_builder.cmd,
//             arguments: p.ability_builder.args,
//             conditions: p.conditions.into_iter().map(|c| c.into()).collect(),
//
//             metadata: p.metadata.merge(),
//             nonce: p.nonce,
//
//             not_before: p.not_before,
//             expiration: p.expiration,
//         }
//     }
// }
