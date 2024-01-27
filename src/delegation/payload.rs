use super::condition::Condition;
use crate::{
    ability::traits::{Command, Delegatable, DynJs},
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

impl<'a, T: ?Sized + Delegatable + Resolvable + Clone, C: Condition> Payload<T, C> {
    pub fn check<U: Delegatable + Clone>(
        invoked: invocation::Payload<T>, // FIXME promiroy version
        proofs: Vec<Payload<U, C>>,
        now: SystemTime,
    ) -> Result<(), ()>
    where
        Ipld: From<T> + From<U::Builder>,
        U: TryProve<U>,
        U::Builder: Clone + Delegatable,
        <U as Delegatable>::Builder: TryProve<U> + TryProve<<U as Delegatable>::Builder>,
        <<U as Delegatable>::Builder as TryProve<<U as Delegatable>::Builder>>::Error: Clone,
        Prev<U>: From<<U as Delegatable>::Builder>,
        Prev<<U as Delegatable>::Builder>: From<<U as Delegatable>::Builder>,
        T: TryProve<U> + TryProve<<U as Delegatable>::Builder> + Clone,
        <T as Delegatable>::Builder: From<invocation::Payload<T>>,
        <U as Delegatable>::Builder: From<<T as Delegatable>::Builder>,
        <<T as Delegatable>::Builder as TryProve<U>>::Error: Clone,
        <T as Delegatable>::Builder: Clone + TryProve<U> + TryProve<U::Builder>,
        <T as TryProve<U::Builder>>::Error: Clone,
        <U as Delegatable>::Builder: TryProve<
            <<U as Delegatable>::Builder as TryProve<<U as Delegatable>::Builder>>::Proven,
        >,
        T::Builder: TryProve<T::Builder>,
    {
        let builder: T::Builder = invoked.into();
        let start: Prev<T::Builder> = Prev {
            issuer: invoked.issuer,
            subject: invoked.subject,
            ability_builder: Box::new(builder),
        };

        let ipld: Ipld = invoked.into();

        //         let result: Result<Prev<T::Builder>, ()> = proofs.iter().fold(Ok(start), |prev, proof| {
        //             if let Ok(to_check) = prev {
        //                 // FIXME check conditions against ipldified invoked
        //                 match step(&to_check, &proof, &ipld, now) {
        //                     Err(_) => Err(()),
        //                     Ok(next) => Ok(Prev {
        //                         issuer: proof.issuer,
        //                         subject: proof.subject,
        //                         ability_builder: Box::new(next),
        //                     }),
        //                 }
        //             } else {
        //                 prev
        //             }
        //         });
        //
        //         match result {
        //             Ok(_) => Ok(()),
        //             Err(_) => Err(()),
        //         }
        todo!()
    }
}

enum Either<A: ?Sized, B: ?Sized> {
    Left(Box<A>),
    Right(Box<B>),
}

// FIXME "CanProve"
trait ProofHack<U: ?Sized> {
    fn try_prove1(&self, proof: U) -> Result<Either<(), U>, ()>;
}

impl<T: ?Sized, U> ProofHack<U> for T
where
    T: TryProve<U>,
{
    fn try_prove1(&self, proof: U) -> Result<Either<(), U>, ()> {
        match self.try_prove(proof) {
            Ok(_) => Ok(Either::Left(Box::new(()))),
            Err(_) => Ok(Either::Right(Box::new(proof))),
        }
    }
}

struct Prev<T: ?Sized> {
    issuer: Did,
    subject: Did,
    ability_builder: Box<dyn ProofHack<T>>,
}

impl<T: Resolvable> From<invocation::Payload<T>> for Prev<T::Builder>
where
    T::Builder: ProofHack<T::Builder>,
{
    fn from(invoked: invocation::Payload<T>) -> Self {
        Prev {
            issuer: invoked.issuer,
            subject: invoked.subject,
            ability_builder: Box::new(invoked.ability.into()),
        }
    }
}

impl<T: Delegatable + Debug, C: Condition> From<Payload<T, C>> for Prev<T::Builder>
where
    T::Builder: ProofHack<T::Builder>,
{
    fn from(delegation: Payload<T, C>) -> Self {
        Prev {
            issuer: delegation.issuer,
            subject: delegation.subject,
            ability_builder: Box::new(delegation.ability_builder),
        }
    }
}

// FIXME this needs to move to Delegatable
fn step<'a, T, U: Delegatable, C: Condition>(
    prev: &'a Prev<T>,
    proof: &'a Payload<U, C>,
    invoked_ipld: &'a Ipld,
    now: SystemTime,
) -> ()
// FIXME
where
    T: TryProve<<U as Delegatable>::Builder> + Clone,
    U::Builder: Clone,
    Ipld: From<U::Builder>,
    <T as TryProve<U::Builder>>::Error: Clone,
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

    Box::leak(prev.ability_builder).try_prove1(proof.ability_builder.clone()); // So many clones that this may as well be owned

    ()
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
