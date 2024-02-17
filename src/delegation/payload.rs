use super::{
    condition::Condition,
    error::{DelegationError, EnvelopeError},
};
use crate::{
    ability::{
        arguments,
        command::{Command, ParseAbility, ToCommand},
    },
    capsule::Capsule,
    did::{Did, Verifiable},
    nonce::Nonce,
    proof::{
        checkable::Checkable,
        parents::CheckParents,
        prove::{Prove, Success},
        same::CheckSame,
    },
    time::{TimeBoundError, Timestamp},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{
    de::{self, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize, Serializer,
};
use std::{collections::BTreeMap, fmt, fmt::Debug};
use web_time::SystemTime;

impl<DID: Did, C: Condition, D> Verifiable<DID> for Payload<D, C, DID> {
    fn verifier(&self) -> &DID {
        &self.issuer
    }
}

/// The payload portion of a [`Delegation`][super::Delegation].
///
/// This contains the semantic information about the delegation, including the
/// issuer, subject, audience, the delegated ability, time bounds, and so on.
#[derive(Debug, Clone, PartialEq)]
pub struct Payload<D, C: Condition, DID: Did> {
    /// The subject of the [`Delegation`].
    ///
    /// This role *must* have issued the earlier (root)
    /// delegation in the chain. This makes the chains
    /// self-certifying.
    ///
    /// The semantics of the delegation are established
    /// by the subject.
    ///
    /// [`Delegation`]: super::Delegation
    pub subject: DID,

    /// The issuer of the [`Delegation`].
    ///
    /// This [`Did`] *must* match the signature on
    /// the outer layer of [`Delegation`].
    ///
    /// [`Delegation`]: super::Delegation
    pub issuer: DID,

    /// The agent being delegated to.
    pub audience: DID,

    /// A delegatable ability chain.
    ///
    /// Note that this should be is some [Proof::Hierarchy]
    pub ability_builder: D,

    /// Any [`Condition`]s on the `ability_builder`.
    pub conditions: Vec<C>,

    /// Extensible, free-form fields.
    pub metadata: BTreeMap<String, Ipld>,

    /// A [cryptographic nonce] to ensure that the UCAN's [`Cid`] is unique.
    ///
    /// [cryptograpgic nonce]: https://en.wikipedia.org/wiki/Cryptographic_nonce
    /// [`Cid`]: libipld_core::cid::Cid ;
    pub nonce: Nonce,

    /// The latest wall-clock time that the UCAN is valid until,
    /// given as a [Unix timestamp].
    ///
    /// [Unix timestamp]: https://en.wikipedia.org/wiki/Unix_time
    pub expiration: Timestamp,

    /// An optional earliest wall-clock time that the UCAN is valid from,
    /// given as a [Unix timestamp].
    ///
    /// [Unix timestamp]: https://en.wikipedia.org/wiki/Unix_time
    pub not_before: Option<Timestamp>,
}

impl<D, C: Condition, DID: Did> Payload<D, C, DID> {
    pub fn map_ability<T>(self, f: impl FnOnce(D) -> T) -> Payload<T, C, DID> {
        Payload {
            issuer: self.issuer,
            subject: self.subject,
            audience: self.audience,
            ability_builder: f(self.ability_builder),
            conditions: self.conditions,
            metadata: self.metadata,
            nonce: self.nonce,
            expiration: self.expiration,
            not_before: self.not_before,
        }
    }

    pub fn map_conditon<T: Condition>(self, f: impl FnMut(C) -> T) -> Payload<D, T, DID> {
        Payload {
            issuer: self.issuer,
            subject: self.subject,
            audience: self.audience,
            ability_builder: self.ability_builder,
            conditions: self.conditions.into_iter().map(f).collect(),
            metadata: self.metadata,
            nonce: self.nonce,
            expiration: self.expiration,
            not_before: self.not_before,
        }
    }

    pub fn check_time(&self, now: SystemTime) -> Result<(), TimeBoundError> {
        if SystemTime::from(self.expiration.clone()) < now {
            return Err(TimeBoundError::Expired);
        }

        if let Some(nbf) = self.not_before.clone() {
            if SystemTime::from(nbf) > now {
                return Err(TimeBoundError::NotYetValid);
            }
        }

        Ok(())
    }
}

impl<D, C: Condition, DID: Did> Capsule for Payload<D, C, DID> {
    const TAG: &'static str = "ucan/d/1.0.0-rc.1";
}

impl<T: CheckSame, C: Condition, DID: Did> CheckSame for Payload<T, C, DID> {
    type Error = <T as CheckSame>::Error;

    fn check_same(&self, proof: &Payload<T, C, DID>) -> Result<(), Self::Error> {
        self.ability_builder.check_same(&proof.ability_builder)
    }
}

impl<T: CheckParents, C: Condition, DID: Did> CheckParents for Payload<T, C, DID> {
    type Parents = Payload<T::Parents, C, DID>;
    type ParentError = <T as CheckParents>::ParentError;

    fn check_parent(&self, proof: &Self::Parents) -> Result<(), Self::ParentError> {
        self.ability_builder.check_parent(&proof.ability_builder)
    }
}

impl<D: Clone + ToCommand, C: Condition + Clone + Serialize, DID: Did + Clone> Serialize
    for Payload<D, C, DID>
where
    Ipld: From<C>,
    arguments::Named<Ipld>: From<D>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let count_nbf = if self.not_before.is_some() { 1 } else { 0 };

        let mut state = serializer.serialize_struct("delegation::Payload", 8 + count_nbf)?;

        state.serialize_field("iss", &self.issuer.clone().into().to_string())?;
        state.serialize_field("sub", &self.subject.clone().into().to_string())?;
        state.serialize_field("aud", &self.audience.clone().into().to_string())?;
        state.serialize_field("meta", &self.metadata)?;
        state.serialize_field("nonce", &self.nonce)?;
        state.serialize_field("exp", &self.expiration)?;

        state.serialize_field("cmd", &self.ability_builder.to_command())?;

        state.serialize_field(
            "args",
            &arguments::Named::from(self.ability_builder.clone()),
        )?;

        state.serialize_field(
            "cond",
            &self
                .conditions
                .iter()
                .map(|c| Ipld::from(c.clone()))
                .collect::<Vec<Ipld>>(),
        )?;

        if let Some(nbf) = self.not_before {
            state.serialize_field("nbf", &nbf)?;
        }

        state.end()
    }
}

impl<
        'de,
        T: ParseAbility + Deserialize<'de> + ToCommand,
        C: Condition + Deserialize<'de>,
        DID: Did + Deserialize<'de>,
    > Deserialize<'de> for Payload<T, C, DID>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DelegationPayloadVisitor<T, C: Condition, DID>(
            std::marker::PhantomData<(T, C, DID)>,
        );

        const FIELDS: &'static [&'static str] = &[
            "iss", "sub", "aud", "cmd", "args", "cond", "meta", "nonce", "exp", "nbf",
        ];

        impl<
                'de,
                T: ParseAbility + Deserialize<'de>,
                C: Condition + Deserialize<'de>,
                DID: Did + Deserialize<'de>,
            > Visitor<'de> for DelegationPayloadVisitor<T, C, DID>
        {
            type Value = Payload<T, C, DID>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct delegation::Payload")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut issuer = None;
                let mut subject = None;
                let mut audience = None;
                let mut command = None;
                let mut arguments = None;
                let mut conditions = None;
                let mut metadata = None;
                let mut nonce = None;
                let mut expiration = None;
                let mut not_before = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "iss" => {
                            if issuer.is_some() {
                                return Err(de::Error::duplicate_field("iss"));
                            }
                            issuer = Some(map.next_value()?);
                        }
                        "sub" => {
                            if subject.is_some() {
                                return Err(de::Error::duplicate_field("sub"));
                            }
                            subject = Some(map.next_value()?);
                        }
                        "aud" => {
                            if audience.is_some() {
                                return Err(de::Error::duplicate_field("aud"));
                            }
                            audience = Some(map.next_value()?);
                        }
                        "cmd" => {
                            if command.is_some() {
                                return Err(de::Error::duplicate_field("cmd"));
                            }
                            command = Some(map.next_value()?);
                        }
                        "args" => {
                            if arguments.is_some() {
                                return Err(de::Error::duplicate_field("args"));
                            }
                            arguments = Some(map.next_value()?);
                        }
                        "cond" => {
                            if conditions.is_some() {
                                return Err(de::Error::duplicate_field("cond"));
                            }
                            conditions = Some(map.next_value()?);
                        }
                        "meta" => {
                            if metadata.is_some() {
                                return Err(de::Error::duplicate_field("meta"));
                            }
                            metadata = Some(map.next_value()?);
                        }
                        "nonce" => {
                            if nonce.is_some() {
                                return Err(de::Error::duplicate_field("nonce"));
                            }
                            nonce = Some(map.next_value()?);
                        }
                        "exp" => {
                            if expiration.is_some() {
                                return Err(de::Error::duplicate_field("exp"));
                            }
                            expiration = Some(map.next_value()?);
                        }
                        "nbf" => {
                            if not_before.is_some() {
                                return Err(de::Error::duplicate_field("nbf"));
                            }
                            not_before = Some(map.next_value()?);
                        }
                        other => {
                            return Err(de::Error::unknown_field(other, FIELDS));
                        }
                    }
                }

                let cmd: String = command.ok_or(de::Error::missing_field("cmd"))?;
                let args = arguments.ok_or(de::Error::missing_field("args"))?;

                let ability_builder =
                    <T as ParseAbility>::try_parse(cmd.as_str(), &args).map_err(|e| {
                        de::Error::custom(format!(
                            "Unable to parse ability field for {:?} because {:?}",
                            cmd, e
                        ))
                    })?;

                Ok(Payload {
                    issuer: issuer.ok_or(de::Error::missing_field("iss"))?,
                    subject: subject.ok_or(de::Error::missing_field("sub"))?,
                    audience: audience.ok_or(de::Error::missing_field("aud"))?,
                    conditions: conditions.ok_or(de::Error::missing_field("cond"))?,
                    metadata: metadata.ok_or(de::Error::missing_field("meta"))?,
                    nonce: nonce.ok_or(de::Error::missing_field("nonce"))?,
                    expiration: expiration.ok_or(de::Error::missing_field("exp"))?,
                    ability_builder,
                    not_before,
                })
            }
        }

        deserializer.deserialize_struct(
            "DelegationPayload",
            FIELDS,
            DelegationPayloadVisitor(Default::default()),
        )
    }
}

impl<
        T: ParseAbility + Command + for<'de> Deserialize<'de>,
        C: Condition + for<'de> Deserialize<'de>,
        DID: Did + for<'de> Deserialize<'de>,
    > TryFrom<Ipld> for Payload<T, C, DID>
{
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl<T, C: Condition, DID: Did> From<Payload<T, C, DID>> for Ipld {
    fn from(payload: Payload<T, C, DID>) -> Self {
        payload.into()
    }
}

impl<T: Checkable + Clone + Into<arguments::Named<Ipld>>, C: Condition, DID: Did + Clone>
    Payload<T, C, DID>
{
    pub fn check(
        &self,
        proofs: Vec<Payload<T::Hierarchy, C, DID>>,
        now: &SystemTime,
    ) -> Result<(), DelegationError<<T::Hierarchy as Prove>::Error>>
    where
        T::Hierarchy: Clone + Into<arguments::Named<Ipld>>,
    {
        let start: Acc<T::Hierarchy, DID> = Acc {
            issuer: self.issuer.clone(),
            subject: self.subject.clone(),
            hierarchy: T::Hierarchy::from(self.ability_builder.clone()),
        };

        let args: arguments::Named<Ipld> = self.ability_builder.clone().into();

        proofs.into_iter().fold(Ok(start), |prev, proof| {
            if let Ok(prev_) = prev {
                prev_.step(&proof, &args, now).map(move |success| {
                    match success {
                        Success::ProvenByAny => Acc {
                            issuer: proof.issuer.clone(),
                            subject: proof.subject.clone(),
                            hierarchy: prev_.hierarchy,
                        },
                        Success::Proven => Acc {
                            issuer: proof.issuer.clone(),
                            subject: proof.subject.clone(),
                            hierarchy: proof.ability_builder.clone(), // FIXME double check
                        },
                    }
                })
            } else {
                prev
            }
        })?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Acc<H: Prove, DID: Did> {
    issuer: DID,
    subject: DID,
    hierarchy: H,
}

impl<H: Prove, DID: Did> Acc<H, DID> {
    // FIXME this should move to Delegable?
    fn step<'a, C: Condition>(
        &self,
        proof: &Payload<H, C, DID>,
        args: &arguments::Named<Ipld>,
        now: &SystemTime,
    ) -> Result<Success, DelegationError<<H as Prove>::Error>>
    where
        H: Prove + Clone + Into<arguments::Named<Ipld>>,
    {
        if self.issuer != proof.audience {
            return Err(EnvelopeError::InvalidSubject.into());
        }

        if self.subject != proof.subject {
            return Err(EnvelopeError::MisalignedIssAud.into());
        }

        if SystemTime::from(proof.expiration.clone()) > *now {
            return Err(EnvelopeError::Expired.into());
        }

        if let Some(nbf) = proof.not_before.clone() {
            if SystemTime::from(nbf) > *now {
                return Err(EnvelopeError::NotYetValid.into());
            }
        }

        // This could be more efficient (dedup) with sets, but floats don't Ord :(
        for c in proof.conditions.iter() {
            // Validate both current & proof integrity.
            // This should have the same semantic guarantees as looking at subsets,
            // but for all known conditions will run much faster on average.
            // Plz let me know if I got this wrong.
            // —@expede
            if !c.validate(&args) || !c.validate(&self.hierarchy.clone().into()) {
                return Err(DelegationError::FailedCondition);
            }
        }

        self.hierarchy
            .check(&proof.ability_builder.clone())
            .map_err(DelegationError::SemanticError)
    }
}

// use crate::proof::{parentful::Parentful, parentless::Parentless};
//
// impl<B: Checkable<Hierarchy = Parentless<B>>, C, DID: Did> Checkable for Payload<B, C, DID> {
//     type Hierarchy = Parentless<Payload<B, C, DID>>;
// }
//
// impl<B: Checkable<Hierarchy = Parentful<B>>, C, DID: Did> Checkable for Payload<B, C, DID> {
//     type Hierarchy = Parentful<Payload<B, C, DID>>;
// }
