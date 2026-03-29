//! Typesafe builder for [`Delegation`].

use super::{policy::predicate::Predicate, subject::DelegatedSubject};
use crate::{
    command::{Command, CommandParseError},
    crypto::nonce::Nonce,
    did::{Did, DidSigner},
    envelope::{Envelope, EnvelopePayload},
    sealed::{CommandOrUnset, DelegatedSubjectOrUnset, DidOrUnset, DidSignerOrUnset},
    time::timestamp::Timestamp,
    unset::Unset,
};
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use core::marker::PhantomData;
use ipld_core::ipld::Ipld;
use varsig::{
    codec::DagCborCodec,
    signer::{Sign, SignerError},
    verify::Verify,
    Varsig,
};

/// A typesafe [`Delegation`] builder.
///
/// All mandatory fields must be set in order for the `try_build` method to
/// become available:
///
/// - `issuer` via [`DelegationBuilder::issuer`]
/// - `audience` via [`DelegationBuilder::audience`]
/// - `subject` via [`DelegationBuilder::subject`]
/// - `command` via [`DelegationBuilder::command`]
#[allow(missing_debug_implementations)]
pub struct DelegationBuilder<
    D: DidSignerOrUnset,
    Audience: DidOrUnset,
    Subject: DelegatedSubjectOrUnset,
    Cmd: CommandOrUnset,
> {
    issuer: D,
    audience: Audience,
    subject: Subject,
    command: Cmd,

    policy: Vec<Predicate>,
    expiration: Option<Timestamp>,
    not_before: Option<Timestamp>,
    meta: BTreeMap<String, Ipld>,
    nonce: Option<Nonce>,
    _marker: PhantomData<(D, Audience, Subject, Cmd)>,
}

impl Default for DelegationBuilder<Unset, Unset, Unset, Unset> {
    fn default() -> Self {
        Self::new()
    }
}

impl DelegationBuilder<Unset, Unset, Unset, Unset> {
    /// Creates a new, empty [`DelegationBuilder`].
    #[must_use]
    pub const fn new() -> Self {
        DelegationBuilder {
            issuer: Unset,
            audience: Unset,
            subject: Unset,
            command: Unset,
            policy: Vec::new(),
            expiration: None,
            not_before: None,
            meta: BTreeMap::new(),
            nonce: None,
            _marker: PhantomData,
        }
    }
}

impl<
        D: DidSignerOrUnset,
        Audience: DidOrUnset,
        Subject: DelegatedSubjectOrUnset,
        Cmd: CommandOrUnset,
    > DelegationBuilder<D, Audience, Subject, Cmd>
{
    /// Sets the issuer of the [`Delegation`].
    pub fn issuer<NewD: DidSigner>(
        self,
        issuer: NewD,
    ) -> DelegationBuilder<NewD, Audience, Subject, Cmd> {
        DelegationBuilder {
            issuer,
            audience: self.audience,
            subject: self.subject,
            command: self.command,
            policy: self.policy,
            expiration: self.expiration,
            not_before: self.not_before,
            meta: self.meta,
            nonce: self.nonce,
            _marker: PhantomData,
        }
    }

    /// Sets the audience of the [`Delegation`].
    pub fn audience<NewAudience: Did>(
        self,
        audience: NewAudience,
    ) -> DelegationBuilder<D, NewAudience, Subject, Cmd> {
        DelegationBuilder {
            issuer: self.issuer,
            audience,
            subject: self.subject,
            command: self.command,
            policy: self.policy,
            expiration: self.expiration,
            not_before: self.not_before,
            meta: self.meta,
            nonce: self.nonce,
            _marker: PhantomData,
        }
    }

    /// Sets the subject of the [`Delegation`].
    pub fn subject<NewSubject: Into<DelegatedSubject<NewDid>>, NewDid: Did>(
        self,
        subject: NewSubject,
    ) -> DelegationBuilder<D, Audience, DelegatedSubject<NewDid>, Cmd> {
        DelegationBuilder {
            issuer: self.issuer,
            audience: self.audience,
            subject: subject.into(),
            command: self.command,
            policy: self.policy,
            expiration: self.expiration,
            not_before: self.not_before,
            meta: self.meta,
            nonce: self.nonce,
            _marker: PhantomData,
        }
    }

    /// Sets the command of the [`Delegation`] from a pre-validated [`Command`].
    pub fn command(self, command: Command) -> DelegationBuilder<D, Audience, Subject, Command> {
        DelegationBuilder {
            issuer: self.issuer,
            audience: self.audience,
            subject: self.subject,
            command,
            policy: self.policy,
            expiration: self.expiration,
            not_before: self.not_before,
            meta: self.meta,
            nonce: self.nonce,
            _marker: PhantomData,
        }
    }

    /// Parses a command string and sets it on the [`Delegation`].
    ///
    /// # Errors
    ///
    /// Returns [`CommandParseError`] if the command string is invalid.
    pub fn command_from_str(
        self,
        s: &str,
    ) -> Result<DelegationBuilder<D, Audience, Subject, Command>, CommandParseError> {
        Ok(self.command(Command::parse(s)?))
    }

    /// Sets the policy of the [`Delegation`].
    #[must_use]
    pub fn policy(self, policy: Vec<Predicate>) -> Self {
        DelegationBuilder { policy, ..self }
    }

    /// Sets the expiration of the [`Delegation`].
    #[must_use]
    pub fn expiration(self, expiration: Timestamp) -> Self {
        DelegationBuilder {
            expiration: Some(expiration),
            ..self
        }
    }

    /// Sets the not-before timestamp of the [`Delegation`].
    #[must_use]
    pub fn not_before(self, not_before: Timestamp) -> Self {
        DelegationBuilder {
            not_before: Some(not_before),
            ..self
        }
    }

    /// Sets the metadata of the [`Delegation`].
    #[must_use]
    pub fn meta(self, meta: BTreeMap<String, Ipld>) -> Self {
        DelegationBuilder { meta, ..self }
    }

    /// Sets the nonce of the [`Delegation`].
    #[must_use]
    pub fn nonce(self, nonce: Nonce) -> Self {
        DelegationBuilder {
            nonce: Some(nonce),
            ..self
        }
    }

    /// Sets the current time as the not-before timestamp of the [`Delegation`].
    #[cfg(feature = "std")]
    #[must_use]
    pub fn issue_now(self) -> Self {
        DelegationBuilder {
            not_before: Some(Timestamp::now()),
            ..self
        }
    }
}

impl<D: DidSigner>
    DelegationBuilder<D, <D as DidSigner>::Did, DelegatedSubject<<D as DidSigner>::Did>, Command>
{
    /// Builds the [`DelegationPayload`] without signing.
    ///
    /// A nonce must either have been provided via [`DelegationBuilder::nonce`],
    /// or the `getrandom` feature must be enabled.
    ///
    /// # Panics
    ///
    /// Panics if no nonce was provided and the `getrandom` feature is enabled
    /// but the CSPRNG fails.
    #[allow(clippy::expect_used)]
    pub fn into_payload(self) -> super::DelegationPayload<D::Did> {
        let nonce = self.nonce.unwrap_or_else(|| {
            #[cfg(feature = "getrandom")]
            {
                Nonce::generate_16().expect("failed to generate nonce")
            }
            #[cfg(not(feature = "getrandom"))]
            {
                panic!("nonce is required: either call .nonce() or enable the `getrandom` feature")
            }
        });

        super::DelegationPayload {
            issuer: self.issuer.did().clone(),
            audience: self.audience,
            subject: self.subject,
            command: self.command,
            policy: self.policy,
            expiration: self.expiration,
            not_before: self.not_before,
            meta: self.meta,
            nonce,
        }
    }

    /// Builds the complete, signed [`Delegation`].
    ///
    /// A nonce must either have been provided via [`DelegationBuilder::nonce`],
    /// or the `getrandom` feature must be enabled.
    ///
    /// # Errors
    ///
    /// * `SignerError` if signing the delegation fails.
    ///
    /// # Panics
    ///
    /// Panics if no nonce was provided and the `getrandom` feature is enabled
    /// but the CSPRNG fails.
    #[allow(clippy::expect_used)]
    #[allow(clippy::type_complexity)]
    pub fn try_build(
        self,
    ) -> Result<
        super::Delegation<D::Did>,
        SignerError<
            <DagCborCodec as varsig::codec::Codec<super::DelegationPayload<D::Did>>>::EncodingError,
            <<D::Did as Did>::VarsigConfig as Sign>::SignError,
        >,
    > {
        let nonce = self.nonce.unwrap_or_else(|| {
            #[cfg(feature = "getrandom")]
            {
                #[allow(clippy::expect_used)]
                Nonce::generate_16().expect("failed to generate nonce")
            }
            #[cfg(not(feature = "getrandom"))]
            {
                panic!("nonce is required: either call .nonce() or enable the `getrandom` feature")
            }
        });

        let payload: super::DelegationPayload<D::Did> = super::DelegationPayload {
            issuer: self.issuer.did().clone(),
            audience: self.audience,
            subject: self.subject,
            command: self.command,
            policy: self.policy,
            expiration: self.expiration,
            not_before: self.not_before,
            meta: self.meta,
            nonce,
        };

        let (sig, _) = self.issuer.did().varsig_config().try_sign(
            &DagCborCodec,
            self.issuer.signer(),
            &payload,
        )?;

        let header: Varsig<
            <D::Did as Did>::VarsigConfig,
            DagCborCodec,
            super::DelegationPayload<D::Did>,
        > = Varsig::new(self.issuer.did().varsig_config().clone(), DagCborCodec);

        let envelope_payload: EnvelopePayload<
            <D::Did as Did>::VarsigConfig,
            super::DelegationPayload<D::Did>,
        > = EnvelopePayload { header, payload };

        #[allow(clippy::type_complexity)]
        let envelope: Envelope<
            <D::Did as Did>::VarsigConfig,
            super::DelegationPayload<D::Did>,
            <<D::Did as Did>::VarsigConfig as Verify>::Signature,
        > = Envelope(sig, envelope_payload);

        Ok(super::Delegation(envelope))
    }
}
