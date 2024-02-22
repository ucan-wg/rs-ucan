use super::promise::Resolvable;
use crate::{
    ability::{arguments, command::ToCommand, parse::ParseAbility},
    capsule::Capsule,
    crypto::Nonce,
    delegation::{self, condition::Condition, Delegable, ValidationError},
    did::{Did, Verifiable},
    proof::{checkable::Checkable, prove::Prove},
    time::{Expired, Timestamp},
};
use libipld_core::{cid::Cid, ipld::Ipld};
use serde::{
    de::{self, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize, Serializer,
};
use std::{collections::BTreeMap, fmt::Debug};
use web_time::SystemTime;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[cfg(feature = "test_utils")]
use crate::ipld;

#[cfg(feature = "test_utils")]
use crate::ipld::cid;

#[derive(Debug, Clone, PartialEq)]
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
    pub proofs: Vec<Cid>,

    /// An optional [`Cid`] of the [`Receipt`] that requested this be invoked.
    ///
    /// This is helpful for provenance of calls.
    ///
    /// [`Receipt`]: crate::receipt::Receipt
    pub cause: Option<Cid>,

    /// Extensible, free-form fields.
    pub metadata: BTreeMap<String, Ipld>,

    /// A [cryptographic nonce] to ensure that the UCAN's [`Cid`] is unique.
    ///
    /// [cryptographic nonce]: https://en.wikipedia.org/wiki/Cryptographic_nonce
    pub nonce: Nonce,

    /// An optional [Unix timestamp] (wall-clock time) at which this [`Invocation`]
    /// was created.
    pub issued_at: Option<Timestamp>,

    /// An optional [Unix timestamp] (wall-clock time) at which this [`Invocation`]
    /// should no longer be executed.
    ///
    /// One way of thinking about this is as a `timeout`. It also guards against
    /// certain types of denial-of-service attacks.
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

    pub fn check<C: Condition + Debug + Clone>(
        self,
        proofs: Vec<&delegation::Payload<<A::Builder as Checkable>::Hierarchy, C, DID>>,
        now: &SystemTime,
    ) -> Result<(), ValidationError<<<A::Builder as Checkable>::Hierarchy as Prove>::Error, C>>
    where
        A: Delegable,
        A::Builder: Clone + Into<arguments::Named<Ipld>>,
        <A::Builder as Checkable>::Hierarchy: Clone + Into<arguments::Named<Ipld>>,
        DID: Clone,
    {
        let builder_payload: delegation::Payload<A::Builder, C, DID> = self.into();
        builder_payload.check(proofs, now)
    }
}

impl<A, DID: Did> Capsule for Payload<A, DID> {
    const TAG: &'static str = "ucan/i/1.0.0-rc.1";
}

impl<A: Delegable, C: Condition, DID: Did + Clone> From<Payload<A, DID>>
    for delegation::Payload<A::Builder, C, DID>
{
    fn from(inv_payload: Payload<A, DID>) -> Self {
        delegation::Payload {
            issuer: inv_payload.issuer,
            subject: inv_payload.subject.clone(),
            audience: inv_payload.audience.unwrap_or(inv_payload.subject),

            ability_builder: A::Builder::from(inv_payload.ability),
            conditions: vec![],

            metadata: inv_payload.metadata,
            nonce: inv_payload.nonce,

            not_before: None,
            expiration: inv_payload
                .expiration
                .unwrap_or(Timestamp::postel(SystemTime::now())),
        }
    }
}

impl<A: ToCommand + Into<Ipld>, DID: Did> From<Payload<A, DID>> for arguments::Named<Ipld> {
    fn from(payload: Payload<A, DID>) -> Self {
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
        ]);

        if let Some(aud) = payload.audience {
            args.insert("aud".into(), aud.into().to_string().into());
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

/// A variant that accepts [`Promise`]s.
///
/// [`Promise`]: crate::invocation::promise::Promise
pub type Promised<A, DID> = Payload<<A as Resolvable>::Promised, DID>;

impl<A, DID: Did> From<Payload<A, DID>> for Ipld {
    fn from(payload: Payload<A, DID>) -> Self {
        payload.into()
    }
}

#[cfg(feature = "test_utils")]
impl<T: Arbitrary + Debug, DID: Did + Arbitrary + 'static> Arbitrary for Payload<T, DID>
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
            prop::collection::vec(cid::Newtype::arbitrary().prop_map(|nt| nt.cid), 0..25),
            Option::<cid::Newtype>::arbitrary().prop_map(|opt_nt| opt_nt.map(|nt| nt.cid)),
            Option::<Timestamp>::arbitrary(),
            Option::<Timestamp>::arbitrary(),
            prop::collection::btree_map(".*", ipld::Newtype::arbitrary(), 0..50).prop_map(|m| {
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
