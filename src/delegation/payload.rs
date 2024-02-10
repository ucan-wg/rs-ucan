use super::{
    condition::Condition,
    error::{DelegationError, EnvelopeError},
};
use crate::{
    ability::{
        arguments,
        command::{Command, ToCommand},
    },
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
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{de::DeserializeOwned, ser::SerializeStruct, Deserialize, Serialize, Serializer};
use std::{collections::BTreeMap, fmt::Debug};
use web_time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<D, C: Condition> {
    pub issuer: Did,
    pub subject: Did,
    pub audience: Did,

    /// A delegatable ability chain.
    ///
    /// Note that this should be is some [Proof::Hierarchy] // FIXME enforce in types?
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

impl<D: Clone + ToCommand, C: Condition + Clone + Serialize> Serialize for Payload<D, C>
where
    Ipld: From<C>,
    arguments::Named<Ipld>: From<D>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("delegation::Payload", 9)?;
        state.serialize_field("iss", &self.issuer)?;
        state.serialize_field("sub", &self.subject)?;
        state.serialize_field("aud", &self.audience)?;
        state.serialize_field("cmd", &self.delegated_ability.to_command())?;
        state.serialize_field(
            "args",
            &arguments::Named::from(self.delegated_ability.clone()),
        )?;

        state.serialize_field(
            "cond",
            &self
                .conditions
                .iter()
                .map(|c| Ipld::from(c.clone()))
                .collect::<Vec<Ipld>>(),
        )?;

        state.serialize_field("meta", &self.metadata)?;
        state.serialize_field("nonce", &self.nonce)?;
        state.serialize_field("exp", &self.expiration)?;
        state.serialize_field("nbf", &self.not_before)?;
        state.end()
    }
}

impl<'de, T, C: Condition + DeserializeOwned> Deserialize<'de> for Payload<T, C>
where
    Payload<T, C>: TryFrom<InternalDeserializer>,
    <Payload<T, C> as TryFrom<InternalDeserializer>>::Error: Debug,
{
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        InternalDeserializer::deserialize(d).and_then(|s| {
            s.try_into()
                .map_err(|e| serde::de::Error::custom(format!("{:?}", e))) // FIXME better error
        })
    }
}

impl<T, C: Condition + Serialize + DeserializeOwned> TryFrom<Ipld> for Payload<T, C>
where
    Payload<T, C>: TryFrom<InternalDeserializer>,
{
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let s: InternalDeserializer = ipld_serde::from_ipld(ipld).map_err(|_| ())?; // FIXME
        s.try_into().map_err(|_| ()) // FIXME
    }
}

impl<T, C: Condition> From<Payload<T, C>> for Ipld {
    fn from(payload: Payload<T, C>) -> Self {
        payload.into()
    }
}

impl<'a, T, C: Condition> Payload<T, C> {
    pub fn check(
        delegated: &'a Payload<T, C>, // FIXME promisory version
        proofs: Vec<Payload<T, C>>,
        now: SystemTime,
    ) -> Result<(), DelegationError<<T as Prove>::Error>>
    where
        T: Checkable + CheckSame + Clone + Prove + Into<arguments::Named<Ipld>>,
    {
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
struct Acc<T: Prove> {
    issuer: Did,
    subject: Did,
    hierarchy: T,
}

// FIXME this should move to Delegatable?
fn step<'a, T: Prove + Clone + Into<arguments::Named<Ipld>>, C: Condition>(
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

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct InternalDeserializer {
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

impl<B: ToCommand + From<arguments::Named<Ipld>>, C: Condition + From<Ipld>>
    TryFrom<InternalDeserializer> for Payload<B, C>
{
    type Error = (); // FIXME

    fn try_from(d: InternalDeserializer) -> Result<Self, Self::Error> {
        let p: Self = Payload {
            issuer: d.issuer,
            subject: d.subject,
            audience: d.audience,

            delegated_ability: d.arguments.try_into().map_err(|_| ())?, // d.command.into(),
            conditions: d.conditions.into_iter().map(|c| c.into()).collect(),

            metadata: d.metadata,
            nonce: d.nonce,

            not_before: d.not_before,
            expiration: d.expiration,
        };

        if p.delegated_ability.to_command() != d.command {
            return Err(());
        }

        Ok(p)
    }
}

impl TryFrom<Ipld> for InternalDeserializer {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}
