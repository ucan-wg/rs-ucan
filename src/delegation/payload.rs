use super::policy::Predicate;
use crate::ability::arguments::Named;
use crate::{
    capsule::Capsule,
    crypto::{varsig, Nonce},
    did::{Did, Verifiable},
    time::{TimeBoundError, Timestamp},
};
use core::str::FromStr;
use derive_builder::Builder;
use libipld_core::{codec::Codec, error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};
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

impl<DID: Did + FromStr> TryFrom<Named<Ipld>> for Payload<DID> {
    type Error = (); // FIXME

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
                    subject = Some(
                        match ipld {
                            Ipld::Null => None,
                            Ipld::String(s) => Some(DID::from_str(s.as_str()).map_err(|_| ())?),
                            _ => return Err(()),
                        }
                        .ok_or(())?,
                    )
                }
                "iss" => match ipld {
                    Ipld::String(s) => issuer = Some(DID::from_str(s.as_str()).map_err(|_| ())?),
                    _ => return Err(()),
                },
                "aud" => match ipld {
                    Ipld::String(s) => audience = Some(DID::from_str(s.as_str()).map_err(|_| ())?),
                    _ => return Err(()),
                },
                "via" => match ipld {
                    Ipld::String(s) => via = Some(DID::from_str(s.as_str()).map_err(|_| ())?),
                    _ => return Err(()),
                },
                "cmd" => match ipld {
                    Ipld::String(s) => command = Some(s),
                    _ => return Err(()),
                },
                "pol" => match ipld {
                    Ipld::List(xs) => {
                        policy = xs
                            .iter()
                            .map(|x| Predicate::try_from(x.clone()).ok())
                            .collect();
                    }
                    _ => return Err(()),
                },
                "metadata" => match ipld {
                    Ipld::Map(m) => metadata = Some(m),
                    _ => return Err(()),
                },
                "nonce" => match ipld {
                    Ipld::Bytes(b) => nonce = Some(Nonce::from(b).into()),
                    _ => return Err(()),
                },
                "exp" => match ipld {
                    Ipld::Integer(i) => expiration = Some(Timestamp::try_from(i).map_err(|_| ())?),
                    _ => return Err(()),
                },
                "nbf" => match ipld {
                    Ipld::Integer(i) => not_before = Some(Timestamp::try_from(i).map_err(|_| ())?),
                    _ => return Err(()),
                },
                _ => (),
            }
        }

        Ok(Payload {
            subject,
            issuer: issuer.ok_or(())?,
            audience: audience.ok_or(())?,
            via,
            command: command.ok_or(())?,
            policy: policy.ok_or(())?,
            metadata: metadata.ok_or(())?,
            nonce: nonce.ok_or(())?,
            expiration: expiration.ok_or(())?,
            not_before,
        })
    }
}

impl<DID: Did> From<Payload<DID>> for Named<Ipld> {
    fn from(payload: Payload<DID>) -> Self {
        let mut args = Named::<Ipld>::from_iter([
            ("iss".to_string(), Ipld::from(payload.issuer.to_string())),
            ("aud".to_string(), payload.audience.to_string().into()),
            ("cmd".to_string(), payload.command.into()),
            (
                "pol".to_string(),
                Ipld::List(payload.policy.into_iter().map(|p| p.into()).collect()),
            ),
            ("nonce".to_string(), payload.nonce.into()),
            ("exp".to_string(), payload.expiration.into()),
        ]);

        if let Some(subject) = payload.subject {
            args.insert("sub".to_string(), Ipld::from(subject.to_string()));
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
            prop::collection::btree_map(".*", ipld::Newtype::arbitrary(), 0..50).prop_map(|m| {
                m.into_iter()
                    .map(|(k, v)| (k, v.0))
                    .collect::<BTreeMap<String, Ipld>>()
            }),
            prop::collection::vec(Predicate::arbitrary_with(pred_args), 0..10),
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
                    }
                },
            )
            .boxed()
    }
}
