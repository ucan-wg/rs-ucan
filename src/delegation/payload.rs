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
use serde::{
    de::{self, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize, Serializer,
};
use std::{collections::BTreeMap, fmt, fmt::Debug};
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

// FIXME
use crate::ability::command::ParseAbility;

impl<
        'de,
        T: ParseAbility + Deserialize<'de> + ToCommand,
        C: Condition + TryFrom<Ipld> + Deserialize<'de>,
    > Deserialize<'de> for Payload<T, C>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DelegationPayloadVisitor<T, C: Condition>(std::marker::PhantomData<(T, C)>);

        const FIELDS: &'static [&'static str] = &[
            "iss", "sub", "aud", "cmd", "args", "cond", "meta", "nonce", "exp", "nbf",
        ];

        impl<
                'de,
                T: Deserialize<'de> + ParseAbility + ToCommand,
                C: Condition + TryFrom<Ipld> + Deserialize<'de>,
            > Visitor<'de> for DelegationPayloadVisitor<T, C>
        {
            type Value = Payload<T, C>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct delegation::Payload")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
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

                let delegated_ability = <T as ParseAbility>::try_parse(cmd.as_str(), &args)
                    .map_err(|e| {
                        de::Error::custom(format!(
                            "Unable to parse ability field for {} because {}",
                            cmd, e
                        ))
                    })?;

                Ok(Payload {
                    issuer: issuer.ok_or(de::Error::missing_field("iss"))?,
                    subject: subject.ok_or(de::Error::missing_field("sub"))?,
                    audience: audience.ok_or(de::Error::missing_field("aud"))?,
                    delegated_ability,
                    conditions: conditions.ok_or(de::Error::missing_field("cond"))?,
                    metadata: metadata.ok_or(de::Error::missing_field("meta"))?,
                    nonce: nonce.ok_or(de::Error::missing_field("nonce"))?,
                    expiration: expiration.ok_or(de::Error::missing_field("exp"))?,
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
