use super::resolvable::Resolvable;
use crate::{
    ability::{
        arguments,
        command::{ParseAbility, ToCommand},
    },
    capsule::Capsule,
    delegation::{self, condition::Condition, error::DelegationError, Delegable},
    did::{Did, Verifiable},
    nonce::Nonce,
    proof::{checkable::Checkable, prove::Prove},
    time::{TimeBoundError, Timestamp},
};
use libipld_core::{cid::Cid, ipld::Ipld};
use serde::{
    de::{self, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize, Serializer,
};
use std::{collections::BTreeMap, fmt::Debug};
use web_time::SystemTime;

impl<DID: Did, T> Verifiable<DID> for Payload<T, DID> {
    fn verifier(&self) -> &DID {
        &self.issuer
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<A, DID: Did> {
    pub issuer: DID,
    pub subject: DID,
    pub audience: Option<DID>,

    pub ability: A,

    pub proofs: Vec<Cid>,
    pub cause: Option<Cid>,
    pub metadata: BTreeMap<String, Ipld>,
    pub nonce: Nonce,

    pub issued_at: Option<Timestamp>,
    pub expiration: Option<Timestamp>, // FIXME this field may not make sense
}

// FIXME cleanup traits
// one idea, because they keep comingup together: put hierarchy and builder on the same
// trair (as associated tyeps) to klet us skip the ::bulder::hierarchy indirection.
//
// This probably means putting the delegation T back to the upper level and bieng explicit about
// the T::Builder in the type
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

    // FIXME err type
    pub fn check_time(&self, now: SystemTime) -> Result<(), TimeBoundError> {
        let ts_now = &Timestamp::postel(now);

        if let Some(ref exp) = self.expiration {
            if exp < ts_now {
                panic!("FIXME")
            }
        }

        Ok(())
    }

    pub fn check<C: Condition>(
        self,
        proofs: Vec<&delegation::Payload<<A::Builder as Checkable>::Hierarchy, C, DID>>,
        now: &SystemTime,
    ) -> Result<(), DelegationError<<<A::Builder as Checkable>::Hierarchy as Prove>::Error>>
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
        let mut field_count = 9;
        if self.audience.is_some() {
            field_count += 1
        };
        if self.issued_at.is_some() {
            field_count += 1
        };
        if self.expiration.is_some() {
            field_count += 1
        };

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

                let ability = <T as ParseAbility>::try_parse(cmd.as_str(), &args).map_err(|e| {
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

/// A variant that accepts [`Promise`]s.
///
/// [`Promise`]: crate::invocation::promise::Promise
pub type Promised<A, DID> = Payload<<A as Resolvable>::Promised, DID>;

impl<A, DID: Did> From<Payload<A, DID>> for Ipld {
    fn from(payload: Payload<A, DID>) -> Self {
        payload.into()
    }
}
