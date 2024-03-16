use super::policy::Predicate;
use crate::ability::arguments::Named;
use crate::time;
use crate::{
    capsule::Capsule,
    crypto::{varsig, Nonce},
    did::{Did, Verifiable},
    time::{TimeBoundError, Timestamp},
};
use core::str::FromStr;
use derive_builder::Builder;
use did_url::DID;
use libipld_core::{codec::Codec, error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};
use thiserror::Error;
use web_time::SystemTime;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[cfg(feature = "test_utils")]
use crate::ipld;

/// The payload portion of a [`Delegation`][super::Delegation].
///
/// This contains the semantic information about the delegation, including the
/// issuer, subject, audience, the delegated ability, time bounds, and so on.
#[derive(Debug, Clone, PartialEq, Builder)] // FIXME Serialize, Deserialize, Builder)]
pub struct Payload<DID: Did> {
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
    pub subject: Option<DID>,

    /// The issuer of the [`Delegation`].
    ///
    /// This [`Did`] *must* match the signature on
    /// the outer layer of [`Delegation`].
    ///
    /// [`Delegation`]: super::Delegation
    pub issuer: DID,

    /// The agent being delegated to.
    pub audience: DID,

    /// A [`Did`] that must be in the delegation chain at invocation time.
    #[builder(default)]
    pub via: Option<DID>,

    /// The command being delegated.
    pub command: String,

    /// Any [`Predicate`] policies that constrain the `args` on an [`Invocation`][crate::invocation::Invocation].
    #[builder(default)]
    pub policy: Vec<Predicate>,

    /// Extensible, free-form fields.
    #[builder(default)]
    pub metadata: BTreeMap<String, Ipld>,

    /// A [cryptographic nonce] to ensure that the UCAN's [`Cid`] is unique.
    ///
    /// [cryptograpgic nonce]: https://en.wikipedia.org/wiki/Cryptographic_nonce
    /// [`Cid`]: libipld_core::cid::Cid ;
    #[builder(default = "Nonce::generate_16(&mut vec![])")]
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
    #[builder(default)]
    pub not_before: Option<Timestamp>,
}

impl<DID: Did> Payload<DID> {
    pub fn check_time(&self, now: SystemTime) -> Result<(), TimeBoundError> {
        let ts_now = &Timestamp::postel(now);

        if &self.expiration < ts_now {
            return Err(TimeBoundError::Expired);
        }

        if let Some(ref nbf) = self.not_before {
            if nbf > ts_now {
                return Err(TimeBoundError::NotYetValid);
            }
        }

        Ok(())
    }
}

impl<DID: Did> Capsule for Payload<DID> {
    const TAG: &'static str = "ucan/d@1.0.0-rc.1";
}

impl<DID: Did> Verifiable<DID> for Payload<DID> {
    fn verifier(&self) -> &DID {
        &self.issuer
    }
}

impl<DID: Did + FromStr> TryFrom<Named<Ipld>> for Payload<DID>
where
    <DID as FromStr>::Err: Debug,
{
    type Error = ParseError<DID>;

    fn try_from(args: Named<Ipld>) -> Result<Self, Self::Error> {
        let mut subject = None;
        let mut issuer = None;
        let mut audience = None;
        let mut via = None;
        let mut command = None;
        let mut policy = None;
        let mut metadata = None;
        let mut nonce = None;
        let mut expiration = None;
        let mut not_before = None;

        for (k, ipld) in args {
            match k.as_str() {
                "sub" => {
                    subject = Some(match ipld {
                        Ipld::Null => None,
                        Ipld::String(s) => {
                            Some(DID::from_str(s.as_str()).map_err(ParseError::DidParseError)?)
                        }
                        bad => return Err(ParseError::WrongTypeForField("sub".to_string(), bad)),
                    })
                }
                "iss" => match ipld {
                    Ipld::String(s) => {
                        issuer = Some(DID::from_str(s.as_str()).map_err(ParseError::DidParseError)?)
                    }
                    bad => return Err(ParseError::WrongTypeForField("iss".to_string(), bad)),
                },
                "aud" => match ipld {
                    Ipld::String(s) => {
                        audience =
                            Some(DID::from_str(s.as_str()).map_err(ParseError::DidParseError)?)
                    }
                    bad => return Err(ParseError::WrongTypeForField("aud".to_string(), bad)),
                },
                "via" => match ipld {
                    Ipld::String(s) => {
                        via = Some(DID::from_str(s.as_str()).map_err(ParseError::DidParseError)?)
                    }
                    bad => return Err(ParseError::WrongTypeForField("via".to_string(), bad)),
                },
                "cmd" => match ipld {
                    Ipld::String(s) => command = Some(s),
                    bad => return Err(ParseError::WrongTypeForField("cmd".to_string(), bad)),
                },
                "pol" => match ipld {
                    Ipld::List(xs) => {
                        policy = xs
                            .iter()
                            .map(|x| Predicate::try_from(x.clone()).ok())
                            .collect();
                    }
                    bad => return Err(ParseError::WrongTypeForField("pol".to_string(), bad)),
                },
                "meta" => match ipld {
                    Ipld::Map(m) => metadata = Some(m),
                    bad => return Err(ParseError::WrongTypeForField("meta".to_string(), bad)),
                },
                "nonce" => match ipld {
                    Ipld::Bytes(b) => nonce = Some(Nonce::from(b).into()),
                    bad => return Err(ParseError::WrongTypeForField("nonce".to_string(), bad)),
                },
                "exp" => match ipld {
                    Ipld::Integer(i) => {
                        expiration = Some(Timestamp::try_from(i).map_err(ParseError::BadTimestamp)?)
                    }
                    bad => return Err(ParseError::WrongTypeForField("exp".to_string(), bad)),
                },
                "nbf" => match ipld {
                    Ipld::Integer(i) => {
                        not_before = Some(Timestamp::try_from(i).map_err(ParseError::BadTimestamp)?)
                    }
                    bad => return Err(ParseError::WrongTypeForField("nbf".to_string(), bad)),
                },
                other => return Err(ParseError::UnknownField(other.to_string())),
            }
        }

        Ok(Payload {
            subject: subject.ok_or(ParseError::MissingSub)?,
            issuer: issuer.ok_or(ParseError::MissingIss)?,
            audience: audience.ok_or(ParseError::MissingAud)?,
            via,
            command: command.ok_or(ParseError::MissingCmd)?,
            policy: policy.ok_or(ParseError::MissingPol)?,
            metadata: metadata.unwrap_or_default(),
            nonce: nonce.ok_or(ParseError::MissingNonce)?,
            expiration: expiration.ok_or(ParseError::MissingExp)?,
            not_before,
        })
    }
}

#[derive(Debug, Error)]
pub enum ParseError<DID: FromStr>
where
    <DID as FromStr>::Err: Debug,
{
    #[error("Unknown field: {0}")]
    UnknownField(String),

    #[error("Missing sub field")]
    MissingSub,

    #[error("Missing iss field")]
    MissingIss,

    #[error("Missing aud field")]
    MissingAud,

    #[error("Missing cmd field")]
    MissingCmd,

    #[error("Missing pol field")]
    MissingPol,

    #[error("Missing nonce field")]
    MissingNonce,

    #[error("Missing exp field")]
    MissingExp,

    #[error("Wrong type for field {0}: {1:?}")]
    WrongTypeForField(String, Ipld),

    #[error("Cannot parse DID")]
    DidParseError(<DID as FromStr>::Err),

    #[error("Cannot parse timestamp: {0}")]
    BadTimestamp(#[from] time::OutOfRangeError),
}

impl<DID: Did> From<Payload<DID>> for Ipld {
    fn from(payload: Payload<DID>) -> Self {
        let named: Named<Ipld> = payload.into();
        Ipld::Map(named.0)
    }
}

impl<DID> TryFrom<Ipld> for Payload<DID>
where
    DID: Did + FromStr,
    <DID as FromStr>::Err: Debug,
{
    type Error = TryFromIpldError<DID>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(map) => {
                let named = Named::<Ipld>(map);
                Payload::try_from(named).map_err(TryFromIpldError::MapParseError)
            }
            _ => Err(TryFromIpldError::NotAMap),
        }
    }
}

#[derive(Debug, Error)]
pub enum TryFromIpldError<DID: FromStr>
where
    <DID as FromStr>::Err: Debug,
{
    NotAMap,
    MapParseError(ParseError<DID>),
}

impl<DID: Did> From<Payload<DID>> for Named<Ipld> {
    fn from(payload: Payload<DID>) -> Self {
        let mut args = Named::<Ipld>::from_iter([
            ("iss".to_string(), Ipld::String(payload.issuer.to_string())),
            (
                "aud".to_string(),
                Ipld::String(payload.audience.to_string()),
            ),
            ("cmd".to_string(), Ipld::String(payload.command)),
            ("pol".to_string(), {
                Ipld::List(payload.policy.into_iter().map(|p| p.into()).collect())
            }),
            ("nonce".to_string(), payload.nonce.into()),
            ("exp".to_string(), payload.expiration.into()),
        ]);

        if let Some(subject) = payload.subject {
            args.insert("sub".to_string(), Ipld::String(subject.to_string()));
        } else {
            args.insert("sub".to_string(), Ipld::Null);
        }

        if let Some(not_before) = payload.not_before {
            args.insert("nbf".to_string(), Ipld::from(not_before));
        }

        if !payload.metadata.is_empty() {
            args.insert("meta".to_string(), Ipld::Map(payload.metadata));
        }

        args
    }
}

#[cfg(feature = "test_utils")]
impl<DID: Did + Arbitrary + 'static> Arbitrary for Payload<DID>
where
    DID::Parameters: Clone,
{
    type Parameters = (DID::Parameters, <Predicate as Arbitrary>::Parameters);
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((did_args, pred_args): Self::Parameters) -> Self::Strategy {
        (
            Option::<DID>::arbitrary(),
            DID::arbitrary_with(did_args.clone()),
            DID::arbitrary_with(did_args),
            String::arbitrary(),
            Nonce::arbitrary(),
            Timestamp::arbitrary(),
            Option::<Timestamp>::arbitrary(),
            prop::collection::btree_map(".*", ipld::Newtype::arbitrary(), 0..5).prop_map(|m| {
                m.into_iter()
                    .map(|(k, v)| (k, v.0))
                    .collect::<BTreeMap<String, Ipld>>()
            }),
            prop::collection::vec(Predicate::arbitrary_with(pred_args), 0..10),
            Option::<DID>::arbitrary(),
        )
            .prop_map(
                |(
                    subject,
                    issuer,
                    audience,
                    command,
                    nonce,
                    expiration,
                    not_before,
                    metadata,
                    policy,
                    via,
                )| {
                    Payload {
                        issuer,
                        subject,
                        audience,
                        command,
                        policy,
                        metadata,
                        nonce,
                        expiration,
                        not_before,
                        via,
                    }
                },
            )
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use pretty_assertions as pretty;
    use proptest::prelude::*;
    use testresult::TestResult;

    mod serialization {
        use super::*;

        #[test_log::test]
        fn test_into_ipld() -> TestResult {
            proptest!(ProptestConfig::with_cases(100), |(payload: Payload<crate::did::preset::Verifier>)| {
                let named: Named<Ipld> = payload.clone().into();
                let sub = named.get("sub".into());

                if let Some(ref subject) = payload.subject {
                    let sub_ipld = &Ipld::String(subject.to_string());
                    pretty::assert_eq!(sub, Some(sub_ipld));
                } else {
                    pretty::assert_eq!(sub, Some(&Ipld::Null));
                }
            });

            // proptest! {
            //     #![proptest_config(ProptestConfig {
            //         cases: 100, .. ProptestConfig::default()
            //     })]

            //     #[test_log::test]
            //     fn test_into_ipld(payload: Payload<crate::did::preset::Verifier>) {
            //         dbg!(payload.clone());

            //         prop_assert_eq!(payload.clone(), payload.clone())
            //     }

            //     // #[test_log::test]
            //     // fn test_roundtrip_ipld() -> TestResult {
            //     //     Ok(())
            //     // }
            // }

            Ok(())
        }

        #[test_log::test]
        fn test_from_ipld() -> TestResult {
            Ok(())
        }

        #[test_log::test]
        fn test_ipld_round_trip() -> TestResult {
            proptest!(ProptestConfig::with_cases(1), |(payload: Payload<crate::did::preset::Verifier>)| {
                let ipld: Ipld = payload.clone().into();
                let parsed = Payload::<crate::did::preset::Verifier>::try_from(ipld);

                dbg!(parsed);

                // assert_matches!(parsed, Ok(payload));
            });

            Ok(())
        }

        #[test_log::test]
        fn test_from_invalid_ipld() -> TestResult {
            Ok(())
        }
    }
}
