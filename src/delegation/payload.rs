use super::{
    condition::Condition,
    delegatable::Delegatable,
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
pub struct Payload<T: Delegatable, C: Condition> {
    pub issuer: Did,
    pub subject: Did,
    pub audience: Did,

    pub ability_builder: T::Builder,
    pub conditions: Vec<C>, // FIXME BTreeSet?
    pub metadata: BTreeMap<String, Ipld>,
    pub nonce: Nonce,

    pub expiration: Timestamp,
    pub not_before: Option<Timestamp>,
}

impl<T: Delegatable, C: Condition> Capsule for Payload<T, C> {
    const TAG: &'static str = "ucan/d/1.0.0-rc.1";
}

impl<T: Delegatable, C: Condition + Serialize> Serialize for Payload<T, C>
where
    InternalSerializer: From<Payload<T, C>>,
    Payload<T, C>: Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = InternalSerializer::from(self.clone());
        Serialize::serialize(&s, serializer)
    }
}

impl<'de, T: Delegatable, C: Condition + DeserializeOwned> Deserialize<'de> for Payload<T, C>
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

impl<T: Delegatable, C: Condition + Serialize + DeserializeOwned> TryFrom<Ipld> for Payload<T, C>
where
    Payload<T, C>: TryFrom<InternalSerializer>,
{
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let s: InternalSerializer = ipld_serde::from_ipld(ipld).map_err(|_| ())?;
        s.try_into().map_err(|_| ()) // FIXME
    }
}

impl<T: Delegatable, C: Condition> From<Payload<T, C>> for Ipld {
    fn from(payload: Payload<T, C>) -> Self {
        payload.into()
    }
}

// FIXME this likely should move to invocation
impl<'a, T: Delegatable, C: Condition> Payload<T, C> {
    pub fn check<U>(
        delegated: &'a Payload<T, C>, // FIXME promisory version
        proofs: Vec<Payload<U, C>>,
        now: SystemTime,
    ) -> Result<(), DelegationError<<<T::Builder as Checkable>::Hierarchy as Prove>::Error>>
    where
        arguments::Named<Ipld>: From<U::Builder>,
        Payload<U, C>: Clone,
        U: Delegatable + Clone,
        U::Builder: Clone,
        T::Builder: Checkable + CheckSame + Clone,
        <T::Builder as Checkable>::Hierarchy: CheckSame
            + From<T::Builder>
            + Clone
            + Into<arguments::Named<Ipld>>
            + From<<U as Delegatable>::Builder>,
    {
        let builder = &delegated.ability_builder;
        let hierarchy = <T::Builder as Checkable>::Hierarchy::from(builder.clone());

        // FIXME this is a task
        let start: Acc<T::Builder> = Acc {
            issuer: delegated.issuer.clone(),
            subject: delegated.subject.clone(),
            hierarchy,
        };

        let args: arguments::Named<Ipld> = delegated.ability_builder.clone().into();

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
                                hierarchy: <T::Builder as Checkable>::Hierarchy::from(
                                    proof.ability_builder.clone(),
                                ), // FIXME double check
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
pub(crate) struct Acc<B: Checkable> {
    issuer: Did,
    subject: Did,
    hierarchy: B::Hierarchy,
}

// FIXME this should move to Delegatable?
pub(crate) fn step<'a, B: Checkable, U: Delegatable, C: Condition>(
    prev: &Acc<B>,
    proof: &Payload<U, C>,
    args: &arguments::Named<Ipld>,
    now: SystemTime,
) -> Result<Success, DelegationError<<B::Hierarchy as Prove>::Error>>
where
    arguments::Named<Ipld>: From<U::Builder>,
    U::Builder: Into<B::Hierarchy> + Clone,
    B::Hierarchy: Clone + Into<arguments::Named<Ipld>>,
{
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
        .check(&proof.ability_builder.clone().into())
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

impl<T: Delegatable + Command, C: Condition + Into<Ipld>> From<Payload<T, C>> for InternalSerializer
where
    BTreeMap<String, Ipld>: From<T::Builder>,
{
    fn from(payload: Payload<T, C>) -> Self {
        InternalSerializer {
            issuer: payload.issuer,
            subject: payload.subject,
            audience: payload.audience,

            command: T::COMMAND.into(),
            arguments: payload.ability_builder.into(),
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
