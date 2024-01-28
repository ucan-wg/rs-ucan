use super::condition::Condition;
use crate::{
    ability::traits::{Command, Delegatable, DynJs, HasChecker, JustCheck},
    capsule::Capsule,
    did::Did,
    nonce::Nonce,
    prove::TryProve,
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
    pub conditions: Vec<C>,

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

impl<T: Delegatable + Debug, C: Condition> From<Payload<T, C>> for Ipld {
    fn from(payload: Payload<T, C>) -> Self {
        payload.into()
    }
}

use crate::{ability::traits::Resolvable, invocation::payload as invocation};

impl<'a, T: Delegatable + Resolvable + HasChecker + Clone, C: Condition> Payload<T, C> {
    pub fn check<U: Delegatable + Clone>(
        invoked: invocation::Payload<T>, // FIXME promisory version
        proofs: Vec<Payload<U, C>>,
        now: SystemTime,
    ) -> Result<(), ()>
    where
        // FIXME so so so broken
        invocation::Payload<T>: Clone,
        T::CheckAs: From<invocation::Payload<T>> + From<U::Builder> + JustCheck<T::CheckAs>,
        U::Builder: Clone,
    {
        let check_chain: T::CheckAs = invoked.clone().into();
        let start: Acc<T> = Acc {
            issuer: invoked.issuer.clone(),
            subject: invoked.subject.clone(),
            check_chain,
        };

        let ipld: Ipld = invoked.into();

        let result = proofs.iter().fold(Ok(&start), |prev, proof| {
            if let Ok(to_check) = prev {
                match step1(&to_check, proof, &ipld, now) {
                    Err(_) => Err(()),
                    Ok(next) => Ok(next),
                }
            } else {
                prev
            }
        });

        todo!()
    }
}

#[derive(Clone)]
struct Acc<T: HasChecker> {
    issuer: Did,
    subject: Did,
    check_chain: T::CheckAs,
}

// FIXME this needs to move to Delegatable
fn step1<'a, T: HasChecker, U: Delegatable, C: Condition>(
    prev: &'a Acc<T>,
    proof: &'a Payload<U, C>,
    invoked_ipld: &'a Ipld,
    now: SystemTime,
) -> Result<&'a Acc<T>, ()>
where
    T::CheckAs: From<U::Builder> + JustCheck<T::CheckAs>,
    U::Builder: Clone,
{
    if prev.issuer != proof.audience {
        todo!()
    }

    if prev.subject != proof.subject {
        todo!()
    }

    if let Some(nbf) = proof.not_before.clone() {
        if SystemTime::from(nbf) > now {
            todo!()
        }
    }

    if SystemTime::from(proof.expiration.clone()) > now {
        todo!()
    }

    // FIXME check the spec
    // if self.conditions != proof.conditions {
    //      return Err(());
    //  }

    proof
        .conditions
        .iter()
        .try_fold((), |_acc, c| {
            if c.validate(&invoked_ipld) {
                Ok(())
            } else {
                Err(())
            }
        })
        .expect("FIXME");

    JustCheck::check(&prev.check_chain, &proof.ability_builder.clone().into());

    todo!()
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

    #[serde(rename = "can")]
    command: String,
    #[serde(rename = "args")]
    arguments: BTreeMap<String, Ipld>,
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

impl<T: Delegatable + Command + Debug, C: Condition + Into<Ipld>> From<Payload<T, C>>
    for InternalSerializer
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

impl<C: Condition + TryFrom<Ipld>> TryFrom<InternalSerializer> for Payload<DynJs, C> {
    type Error = (); // FIXME

    fn try_from(s: InternalSerializer) -> Result<Payload<DynJs, C>, ()> {
        Ok(Payload {
            issuer: s.issuer,
            subject: s.subject,
            audience: s.audience,

            ability_builder: DynJs {
                cmd: s.command,
                args: s.arguments,
            },
            conditions: s
                .conditions
                .iter()
                .try_fold(Vec::new(), |mut acc, c| {
                    C::try_from(c.clone()).map(|x| {
                        acc.push(x);
                        acc
                    })
                })
                .map_err(|_| ())?, // FIXME better error (collect all errors

            metadata: s.metadata,
            nonce: s.nonce,

            not_before: s.not_before,
            expiration: s.expiration,
        })
    }
}

impl<C: Condition + Into<Ipld>> From<Payload<DynJs, C>> for InternalSerializer {
    fn from(p: Payload<DynJs, C>) -> Self {
        InternalSerializer {
            issuer: p.issuer,
            subject: p.subject,
            audience: p.audience,

            command: p.ability_builder.cmd,
            arguments: p.ability_builder.args,
            conditions: p.conditions.into_iter().map(|c| c.into()).collect(),

            metadata: p.metadata,
            nonce: p.nonce,

            not_before: p.not_before,
            expiration: p.expiration,
        }
    }
}
