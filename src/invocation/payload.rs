use super::promise::Resolvable;
use crate::ability::command::Command;
use crate::ability::parse::ParseAbilityError;
use crate::delegation::policy::selector;
use crate::invocation::Named;
use crate::time;
use crate::{
    ability::{arguments, command::ToCommand, parse::ParseAbility},
    capsule::Capsule,
    crypto::{varsig, Nonce},
    delegation::{
        self,
        policy::{selector::SelectorError, Predicate},
    },
    did::{Did, Verifiable},
    time::{Expired, Timestamp},
};
use derive_builder::Builder;
use libipld_core::{cid::Cid, codec::Codec, ipld::Ipld};
use serde::{
    de::{self, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize, Serializer,
};
use std::collections::BTreeSet;
use std::str::FromStr;
use std::{collections::BTreeMap, fmt};
use thiserror::Error;
use web_time::SystemTime;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[cfg(feature = "test_utils")]
use crate::ipld;

#[cfg(feature = "test_utils")]
use crate::ipld::cid;

#[derive(Debug, Clone, PartialEq, Builder)]
pub struct Payload<A, DID: Did> {
    /// The subject of the [`Invocation`].
    ///
    /// This is typically also the `audience`, hence the [`audence`]
    /// field is optional.
    ///
    /// This role *must* have issued the earlier (root)
    /// delegation in the chain. This makes the chains
    /// self-certifying.
    ///
    /// The semantics of the delegation are established
    /// by the subject.
    ///
    /// [`Invocation`]: super::Invocation
    pub subject: DID,

    /// The issuer of the [`Invocation`].
    ///
    /// This [`Did`] *must* match the signature on
    /// the outer layer of [`Invocation`].
    ///
    /// [`Invocation`]: super::Invocation
    pub issuer: DID,

    /// The agent being delegated to.
    ///
    /// Note that if this is the same as the [`subject`],
    /// this field may be omitted.
    #[builder(default)]
    pub audience: Option<DID>,

    /// The [Ability] being invoked.
    ///
    /// The specific shape and semantics of this ability
    /// are established by the [`subject`] and the `A` type.
    ///
    /// [Ability]: crate::ability
    pub ability: A,

    /// [`Cid`] links to the proofs that authorize this [`Invocation`].
    ///
    /// These must be given in order starting from one where the [`issuer`]
    /// of this invocation matches the [`audience`] of that [`Delegation`] proof.
    ///
    /// [`Invocation`]: super::Invocation
    /// [`Delegation`]: crate::delegation::Delegation
    #[builder(default)]
    pub proofs: Vec<Cid>,

    /// An optional [`Cid`] of the [`Receipt`] that requested this be invoked.
    ///
    /// This is helpful for provenance of calls.
    ///
    /// [`Receipt`]: crate::receipt::Receipt
    #[builder(default)]
    pub cause: Option<Cid>,

    /// Extensible, free-form fields.
    #[builder(default)]
    pub metadata: BTreeMap<String, Ipld>,

    /// A [cryptographic nonce] to ensure that the UCAN's [`Cid`] is unique.
    ///
    /// [cryptographic nonce]: https://en.wikipedia.org/wiki/Cryptographic_nonce
    #[builder(default = "Nonce::generate_16(&mut vec![])")]
    pub nonce: Nonce,

    /// An optional [Unix timestamp] (wall-clock time) at which this [`Invocation`]
    /// was created.
    #[builder(default)]
    pub issued_at: Option<Timestamp>,

    /// An optional [Unix timestamp] (wall-clock time) at which this [`Invocation`]
    /// should no longer be executed.
    ///
    /// One way of thinking about this is as a `timeout`. It also guards against
    /// certain types of denial-of-service attacks.
    #[builder(default = "Some(Timestamp::five_minutes_from_now())")]
    pub expiration: Option<Timestamp>,
}

impl<A, DID: Did> Payload<A, DID> {
    pub fn map_ability<F, Z>(self, f: F) -> Payload<Z, DID>
    where
        F: FnOnce(A) -> Z,
    {
        Payload {
            issuer: self.issuer,
            subject: self.subject,
            audience: self.audience,
            ability: f(self.ability),
            proofs: self.proofs,
            cause: self.cause,
            metadata: self.metadata,
            nonce: self.nonce,
            issued_at: self.issued_at,
            expiration: self.expiration,
        }
    }

    pub fn check_time(&self, now: SystemTime) -> Result<(), Expired> {
        let ts_now = &Timestamp::postel(now);

        if let Some(ref exp) = self.expiration {
            if exp < ts_now {
                return Err(Expired);
            }
        }

        Ok(())
    }

    pub fn check(
        &self,
        proofs: Vec<&delegation::Payload<DID>>,
        now: SystemTime,
    ) -> Result<(), ValidationError<DID>>
    where
        A: ToCommand + Clone,
        DID: Clone,
        arguments::Named<Ipld>: From<A>,
    {
        let now_ts = Timestamp::postel(now);

        if let Some(exp) = self.expiration {
            if exp < now_ts {
                return Err(ValidationError::Expired);
            }
        }

        let args: arguments::Named<Ipld> = self.ability.clone().into();

        let mut cmd = self.ability.to_command();
        if !cmd.ends_with('/') {
            cmd.push('/');
        }

        let (final_iss, vias) = proofs.into_iter().try_fold(
            (&self.issuer, BTreeSet::new()),
            |(iss, mut vias), proof| {
                if *iss != proof.audience {
                    return Err(ValidationError::MisalignedIssAud.into());
                }

                if let Some(proof_subject) = &proof.subject {
                    if self.subject != *proof_subject {
                        return Err(ValidationError::InvalidSubject.into());
                    }
                }

                if proof.expiration < now_ts {
                    return Err(ValidationError::Expired.into());
                }

                if let Some(nbf) = proof.not_before.clone() {
                    if nbf > now_ts {
                        return Err(ValidationError::NotYetValid.into());
                    }
                }

                vias.remove(&iss);
                if let Some(via_did) = &proof.via {
                    vias.insert(via_did);
                }

                if !cmd.starts_with(&proof.command) {
                    return Err(ValidationError::CommandMismatch(proof.command.clone()));
                }

                let ipld_args = Ipld::from(args.clone());

                for predicate in proof.policy.iter() {
                    if !predicate
                        .clone()
                        .run(&ipld_args)
                        .map_err(ValidationError::SelectorError)?
                    {
                        return Err(ValidationError::FailedPolicy(predicate.clone()));
                    }
                }

                Ok((&proof.issuer, vias))
            },
        )?;

        if self.subject != *final_iss {
            return Err(ValidationError::DidNotTerminateInSubject);
        }

        if !vias.is_empty() {
            return Err(ValidationError::UnfulfilledViaConstraint(
                vias.into_iter().cloned().collect(),
            ));
        }

        Ok(())
    }
}

/// Delegation validation errors.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ValidationError<DID: Did> {
    #[error("The subject of the delegation is invalid")]
    InvalidSubject,

    #[error("The issuer and audience of the delegation are misaligned")]
    MisalignedIssAud,

    #[error("The delegation has expired")]
    Expired,

    #[error("The delegation is not yet valid")]
    NotYetValid,

    #[error("The command of the delegation does not match the proof: {0:?}")]
    CommandMismatch(String),

    #[error("The delegation failed a policy predicate: {0:?}")]
    FailedPolicy(Predicate),

    #[error(transparent)]
    SelectorError(#[from] SelectorError),

    #[error("via field constraint was unfulfilled: {0:?}")]
    UnfulfilledViaConstraint(BTreeSet<DID>),

    #[error("The chain did not terminate in the expected subject")]
    DidNotTerminateInSubject,
}

impl<A, DID: Did> Capsule for Payload<A, DID> {
    const TAG: &'static str = "ucan/i@1.0.0-rc.1";
}

impl<A: ToCommand, DID: Did> From<Payload<A, DID>> for arguments::Named<Ipld>
where
    arguments::Named<Ipld>: From<A>,
{
    fn from(payload: Payload<A, DID>) -> Self {
        let mut args = arguments::Named::from_iter([
            ("iss".into(), { payload.issuer.to_string().into() }),
            ("sub".into(), { payload.subject.to_string().into() }),
            ("cmd".into(), { payload.ability.to_command().into() }),
            ("args".into(), {
                Ipld::Map(arguments::Named::<Ipld>::from(payload.ability).0)
            }),
            ("prf".into(), {
                Ipld::List(payload.proofs.iter().map(Into::into).collect())
            }),
            ("nonce".into(), payload.nonce.into()),
        ]);

        if let Some(aud) = payload.audience {
            args.insert("aud".into(), aud.to_string().into());
        }

        if let Some(cause) = payload.cause {
            args.insert("cause".into(), cause.into());
        }

        if !payload.metadata.is_empty() {
            args.insert("meta".into(), payload.metadata.into());
        }

        if let Some(iat) = payload.issued_at {
            args.insert("iat".into(), iat.into());
        }

        if let Some(exp) = payload.expiration {
            args.insert("exp".into(), exp.into());
        }

        args
    }
}

impl<A: ToCommand, DID: Did> From<Payload<A, DID>> for Ipld
where
    arguments::Named<Ipld>: From<Payload<A, DID>>,
{
    fn from(payload: Payload<A, DID>) -> Self {
        arguments::Named::from(payload).into()
    }
}

impl<A, DID> Serialize for Payload<A, DID>
where
    A: ToCommand + Into<Ipld> + Serialize,
    DID: Did + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let field_count = 9
            + self.audience.is_some() as usize
            + self.issued_at.is_some() as usize
            + self.expiration.is_some() as usize;

        let mut state = serializer.serialize_struct("invocation::Payload", field_count)?;

        state.serialize_field("iss", &self.issuer)?;
        state.serialize_field("sub", &self.subject)?;

        state.serialize_field("cmd", &self.ability.to_command())?;
        state.serialize_field("args", &self.ability)?;

        state.serialize_field("prf", &self.proofs)?;
        state.serialize_field("nonce", &self.nonce)?;
        state.serialize_field("cause", &self.cause)?;
        state.serialize_field("meta", &self.metadata)?;

        if let Some(aud) = &self.audience {
            state.serialize_field("aud", aud)?;
        }

        if let Some(iat) = &self.issued_at {
            state.serialize_field("iat", iat)?;
        }

        if let Some(exp) = &self.expiration {
            state.serialize_field("exp", &exp)?;
        }

        state.end()
    }
}

impl<'de, A: ParseAbility + Deserialize<'de>, DID: Did + Deserialize<'de>> Deserialize<'de>
    for Payload<A, DID>
{
    fn deserialize<D>(deserializer: D) -> Result<Payload<A, DID>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct InvocationPayloadVisitor<A, DID>(std::marker::PhantomData<(A, DID)>);

        const FIELDS: &'static [&'static str] = &[
            "iss", "sub", "aud", "cmd", "args", "prf", "nonce", "cause", "meta", "iat", "exp",
        ];

        impl<'de, T: ParseAbility + Deserialize<'de>, DID: Did + Deserialize<'de>> Visitor<'de>
            for InvocationPayloadVisitor<T, DID>
        {
            type Value = Payload<T, DID>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct invocation::Payload")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut issuer = None;
                let mut subject = None;
                let mut audience = None;
                let mut command = None;
                let mut arguments = None;
                let mut proofs = None;
                let mut nonce = None;
                let mut cause = None;
                let mut metadata = None;
                let mut issued_at = None;
                let mut expiration = None;

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
                            audience = map.next_value()?;
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
                        "prf" => {
                            if proofs.is_some() {
                                return Err(de::Error::duplicate_field("prf"));
                            }
                            proofs = Some(map.next_value()?);
                        }
                        "nonce" => {
                            if nonce.is_some() {
                                return Err(de::Error::duplicate_field("nonce"));
                            }
                            nonce = Some(map.next_value()?);
                        }
                        "cause" => {
                            if cause.is_some() {
                                return Err(de::Error::duplicate_field("cause"));
                            }
                            cause = map.next_value()?;
                        }
                        "meta" => {
                            if metadata.is_some() {
                                return Err(de::Error::duplicate_field("meta"));
                            }
                            metadata = Some(map.next_value()?);
                        }
                        "issued_at" => {
                            if issued_at.is_some() {
                                return Err(de::Error::duplicate_field("iat"));
                            }
                            issued_at = map.next_value()?;
                        }
                        "exp" => {
                            if expiration.is_some() {
                                return Err(de::Error::duplicate_field("exp"));
                            }
                            expiration = map.next_value()?;
                        }
                        other => {
                            return Err(de::Error::unknown_field(other, FIELDS));
                        }
                    }
                }

                let cmd: String = command.ok_or(de::Error::missing_field("cmd"))?;
                let args = arguments.ok_or(de::Error::missing_field("args"))?;

                let ability = <T as ParseAbility>::try_parse(cmd.as_str(), args).map_err(|e| {
                    de::Error::custom(format!(
                        "Unable to parse ability field for {:?} becuase {:?}",
                        cmd, e
                    ))
                })?;

                Ok(Payload {
                    issuer: issuer.ok_or(de::Error::missing_field("iss"))?,
                    subject: subject.ok_or(de::Error::missing_field("sub"))?,
                    proofs: proofs.ok_or(de::Error::missing_field("prf"))?,
                    metadata: metadata.ok_or(de::Error::missing_field("meta"))?,
                    nonce: nonce.ok_or(de::Error::missing_field("nonce"))?,
                    audience,
                    ability,
                    cause,
                    issued_at,
                    expiration,
                })
            }
        }

        deserializer.deserialize_struct(
            "invocation::Payload",
            FIELDS,
            InvocationPayloadVisitor(Default::default()),
        )
    }
}

impl<DID: Did, T> Verifiable<DID> for Payload<T, DID> {
    fn verifier(&self) -> &DID {
        &self.issuer
    }
}

impl<A: ParseAbility, DID: Did> TryFrom<arguments::Named<Ipld>> for Payload<A, DID>
where
    <A as ParseAbility>::ArgsErr: fmt::Debug,
    <DID as FromStr>::Err: fmt::Debug,
{
    type Error = ParseError<A, DID>;

    fn try_from(named: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        let mut subject = None;
        let mut issuer = None;
        let mut audience = None;
        let mut command = None;
        let mut args = None;
        let mut cause = None;
        let mut metadata = None;
        let mut nonce = None;
        let mut expiration = None;
        let mut proofs = None;
        let mut issued_at = None;

        for (k, v) in named {
            match k.as_str() {
                "sub" => {
                    subject = Some(match v {
                        Ipld::String(s) => {
                            DID::from_str(s.as_str()).map_err(ParseError::DidParseError)?
                        }
                        _ => return Err(ParseError::WrongTypeForField(k, v)),
                    })
                }
                "iss" => match v {
                    Ipld::String(s) => {
                        issuer = Some(DID::from_str(s.as_str()).map_err(ParseError::DidParseError)?)
                    }
                    _ => return Err(ParseError::WrongTypeForField(k, v)),
                },
                "aud" => match v {
                    Ipld::String(s) => {
                        audience =
                            Some(DID::from_str(s.as_str()).map_err(ParseError::DidParseError)?)
                    }
                    _ => return Err(ParseError::WrongTypeForField(k, v)),
                },
                "cmd" => match v {
                    Ipld::String(s) => command = Some(s),
                    _ => return Err(ParseError::WrongTypeForField(k, v)),
                },
                "args" => match v.try_into() {
                    Ok(a) => args = Some(a),
                    _ => return Err(ParseError::ArgsNotAMap),
                },
                "meta" => match v {
                    Ipld::Map(m) => metadata = Some(m),
                    _ => return Err(ParseError::WrongTypeForField(k, v)),
                },
                "nonce" => match v {
                    Ipld::Bytes(b) => nonce = Some(Nonce::from(b)),
                    _ => return Err(ParseError::WrongTypeForField(k, v)),
                },
                "cause" => match v {
                    Ipld::Link(c) => cause = Some(c),
                    _ => return Err(ParseError::WrongTypeForField(k, v)),
                },
                "exp" => match v {
                    Ipld::Integer(i) => expiration = Some(i.try_into()?),
                    _ => return Err(ParseError::WrongTypeForField(k, v)),
                },
                "iat" => match v {
                    Ipld::Integer(i) => issued_at = Some(i.try_into()?),
                    _ => return Err(ParseError::WrongTypeForField(k, v)),
                },
                "prf" => match &v {
                    Ipld::List(xs) => {
                        proofs = Some(
                            xs.iter()
                                .map(|x| match x {
                                    Ipld::Link(cid) => Ok(*cid),
                                    _ => Err(ParseError::WrongTypeForField(k.clone(), v.clone())),
                                })
                                .collect::<Result<Vec<Cid>, ParseError<A, DID>>>()?,
                        )
                    }
                    _ => return Err(ParseError::WrongTypeForField(k, v)),
                },
                _ => return Err(ParseError::UnknownField(k.to_string())),
            }
        }

        let cmd = command.ok_or(ParseError::MissingCmd)?;
        let some_args = args.ok_or(ParseError::MissingArgs)?;
        let ability = <A as ParseAbility>::try_parse(cmd.as_str(), some_args)
            .map_err(|e| ParseError::AbilityError(e))?;

        Ok(Payload {
            issuer: issuer.ok_or(ParseError::MissingIss)?,
            subject: subject.ok_or(ParseError::MissingSub)?,
            audience,
            ability,
            proofs: proofs.ok_or(ParseError::MissingProofsField)?,
            cause,
            metadata: metadata.unwrap_or_default(),
            nonce: nonce.ok_or(ParseError::MissingNonce)?,
            issued_at,
            expiration,
        })
    }
}

#[derive(Debug, Error)]
pub enum ParseError<A: ParseAbility, DID: FromStr>
where
    <A as ParseAbility>::ArgsErr: fmt::Debug,
    <DID as FromStr>::Err: fmt::Debug,
{
    #[error("Unknown field: {0}")]
    UnknownField(String),

    #[error("Missing sub field")]
    MissingSub,

    #[error("Missing iss field")]
    MissingIss,

    #[error("Missing cmd field")]
    MissingCmd,

    #[error("Missing args field")]
    MissingArgs,

    #[error("Unable to parse ability: {0:?}")]
    AbilityError(ParseAbilityError<<A as ParseAbility>::ArgsErr>),

    #[error("Missing nonce field")]
    MissingNonce,

    #[error("Wrong type for field {0}: {1:?}")]
    WrongTypeForField(String, Ipld),

    #[error("Cannot parse DID")]
    DidParseError(<DID as FromStr>::Err),

    // FIXME
    #[error("Cannot parse timestamp: {0}")]
    BadTimestamp(#[from] time::OutOfRangeError),

    #[error("Args are not a map")]
    ArgsNotAMap,

    #[error("Misisng proofs field")]
    MissingProofsField,
}

/// A variant that accepts [`Promise`]s.
///
/// [`Promise`]: crate::invocation::promise::Promise
pub type Promised<A, DID> = Payload<<A as Resolvable>::Promised, DID>;

#[cfg(feature = "test_utils")]
impl<T: Arbitrary + fmt::Debug, DID: Did + Arbitrary + 'static> Arbitrary for Payload<T, DID>
where
    T::Strategy: 'static,
    DID::Parameters: Clone,
{
    type Parameters = (T::Parameters, DID::Parameters);
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((t_args, did_args): Self::Parameters) -> Self::Strategy {
        (
            T::arbitrary_with(t_args),
            DID::arbitrary_with(did_args.clone()),
            DID::arbitrary_with(did_args.clone()),
            Option::<DID>::arbitrary_with((0.5.into(), did_args)),
            Nonce::arbitrary(),
            prop::collection::vec(cid::Newtype::arbitrary().prop_map(|nt| nt.cid), 0..12),
            Option::<cid::Newtype>::arbitrary().prop_map(|opt_nt| opt_nt.map(|nt| nt.cid)),
            Option::<Timestamp>::arbitrary(),
            Option::<Timestamp>::arbitrary(),
            prop::collection::btree_map(".*", ipld::Newtype::arbitrary(), 0..12).prop_map(|m| {
                m.into_iter()
                    .map(|(k, v)| (k, v.0))
                    .collect::<BTreeMap<String, Ipld>>()
            }),
        )
            .prop_map(
                |(
                    ability,
                    issuer,
                    subject,
                    audience,
                    nonce,
                    proofs,
                    cause,
                    expiration,
                    issued_at,
                    metadata,
                )| {
                    Payload {
                        issuer,
                        subject,
                        audience,
                        ability,
                        proofs,
                        cause,
                        nonce,
                        metadata,
                        issued_at,
                        expiration,
                    }
                },
            )
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ability::msg::Msg;
    use crate::ipld;
    use assert_matches::assert_matches;
    use pretty_assertions as pretty;
    use proptest::prelude::*;
    use testresult::TestResult;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test_log::test]
        fn test_ipld_round_trip(payload in Payload::<Msg, crate::did::preset::Verifier>::arbitrary()) {
            let observed: Named<Ipld> = payload.clone().into();
            let parsed = Payload::<Msg, crate::did::preset::Verifier>::try_from(observed.clone());

            prop_assert!(parsed.is_ok());
            prop_assert_eq!(parsed.unwrap(), payload);
        }

        #[test_log::test]
        fn test_ipld_only_has_correct_fields(payload in Payload::<Msg, crate::did::preset::Verifier>::arbitrary()) {
            let observed: Ipld = payload.clone().into();

            if let Ipld::Map(named) = observed {
                prop_assert!(named.len() >= 6);
                prop_assert!(named.len() <= 11);

                for key in named.keys() {
                    prop_assert!(matches!(key.as_str(), "sub" | "iss" | "aud" | "cmd" | "args" | "prf" | "cause" | "meta" | "nonce" | "exp" | "iat"));
                }
            } else {
                prop_assert!(false, "ipld map");
            }
        }

         #[test_log::test]
         fn test_ipld_field_types(payload in Payload::<Msg, crate::did::preset::Verifier>::arbitrary()) {
             let named: Named<Ipld> = payload.clone().into();

             let sub = named.get("sub".into());
             let iss = named.get("iss".into());
             let cmd = named.get("cmd".into());
             let args = named.get("args".into());
             let prf = named.get("prf".into());
             let nonce = named.get("nonce".into());

             // Required Fields
             prop_assert_eq!(sub.unwrap(), &Ipld::String(payload.subject.to_string()));
             prop_assert_eq!(iss.unwrap(), &Ipld::String(payload.issuer.to_string()));
             prop_assert_eq!(cmd.unwrap(), &Ipld::String(payload.ability.to_command()));

             prop_assert_eq!(args.unwrap(), &payload.ability.into());
             prop_assert!(matches!(args, Some(Ipld::Map(_))));

             prop_assert!(matches!(prf.unwrap(), &Ipld::List(_)));
             if let Some(Ipld::List(ipld_proofs)) = prf {
                 prop_assert_eq!(ipld_proofs.len(), payload.proofs.len());

                 for entry in ipld_proofs {
                     prop_assert!(matches!(entry, Ipld::Link(_)));
                 }
             } else {
                 prop_assert!(false);
             }

             prop_assert_eq!(nonce.unwrap(), &payload.nonce.into());

             // Optional Fields
             prop_assert_eq!(payload.audience.map(|did| did.into()), named.get("aud").cloned());
             prop_assert_eq!(payload.cause.map(Ipld::Link), named.get("cause").cloned());

             match (payload.metadata.is_empty(), named.get("meta")) {
                 (false, Some(Ipld::Map(btree))) => {
                     prop_assert_eq!(&payload.metadata, btree);
                 }
                 (true, None) => prop_assert!(true),
                 _ => prop_assert!(false)
             }

             match (payload.expiration, named.get("exp")) {
                 (Some(exp), Some(Ipld::Integer(i))) => {
                     prop_assert_eq!(i128::from(exp), i.clone());
                 }
                 (None, None) => prop_assert!(true),
                 _ => prop_assert!(false)
             }

             match (payload.issued_at, named.get("iat")) {
                 (Some(iat), Some(Ipld::Integer(i))) => {
                     prop_assert_eq!(i128::from(iat), i.clone());
                 }
                 (None, None) => prop_assert!(true),
                 _ => prop_assert!(false)
             }
         }

         #[test_log::test]
         fn test_non_payload(named in arguments::Named::<Ipld>::arbitrary()) {
             // Just ensuring that a negative test shows up
             let parsed = Payload::<Msg, crate::did::preset::Verifier>::try_from(named);
             prop_assert!(parsed.is_err())
         }
    }
}
