use super::policy::Predicate;
use crate::{
    capsule::Capsule,
    crypto::Nonce,
    did::{Did, Verifiable},
    time::{TimeBoundError, Timestamp},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    /// The command being delegated.
    pub command: String,

    /// Any [`Predicate`] policies that constrain the `args` on an [`Invocation`][crate::invocation::Invocation].
    pub policy: Vec<Predicate>,

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

impl<DID: Did + for<'de> Deserialize<'de>> TryFrom<Ipld> for Payload<DID> {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl<DID: Did> From<Payload<DID>> for Ipld {
    fn from(payload: Payload<DID>) -> Self {
        payload.into()
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
