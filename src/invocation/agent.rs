use super::{
    payload::{Payload, ValidationError},
    promise::Resolvable,
    store::Store,
    Invocation,
};
use crate::{
    ability::{arguments, parse::ParseAbilityError, ucan::revoke::Revoke},
    crypto::{signature, varsig, Nonce},
    delegation,
    did::Did,
    invocation::promise,
    time::Timestamp,
};
use libipld_core::{
    cid::Cid,
    codec::{Codec, Encode},
    ipld::Ipld,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    marker::PhantomData,
};
use thiserror::Error;
use web_time::SystemTime;

#[derive(Debug)]
pub struct Agent<
    'a,
    T: Resolvable,
    DID: Did,
    S: Store<T::Promised, DID, V, Enc>,
    P: promise::Store<T, DID>,
    D: delegation::store::Store<DID, V, Enc>,
    V: varsig::Header<Enc>,
    Enc: Codec + Into<u32> + TryFrom<u32>,
> {
    /// The agent's [`DID`].
    pub did: &'a DID,

    /// A [`delegation::Store`][delegation::store::Store].
    pub delegation_store: &'a mut D,

    /// A [`Store`][Store] for the agent's [`Invocation`]s.
    pub invocation_store: &'a mut S,

    /// A [`promise::Store`] for the agent's unresolved promises.
    pub unresolved_promise_index: &'a mut P,

    signer: &'a <DID as Did>::Signer,
    marker: PhantomData<(T, V, Enc)>,
}

impl<'a, T, DID, S, P, D, V, Enc> Agent<'a, T, DID, S, P, D, V, Enc>
where
    T::Promised: Clone,
    Ipld: Encode<Enc>,
    delegation::Payload<DID>: Clone,
    T: Resolvable + Clone,
    DID: Did + Clone,
    S: Store<T::Promised, DID, V, Enc>,
    P: promise::Store<T, DID>,
    D: delegation::store::Store<DID, V, Enc>,
    V: varsig::Header<Enc>,
    Enc: Codec + Into<u32> + TryFrom<u32>,
{
    pub fn new(
        did: &'a DID,
        signer: &'a <DID as Did>::Signer,
        invocation_store: &'a mut S,
        delegation_store: &'a mut D,
        unresolved_promise_index: &'a mut P,
    ) -> Self {
        Self {
            did,
            invocation_store,
            delegation_store,
            unresolved_promise_index,
            signer,
            marker: PhantomData,
        }
    }

    pub fn invoke(
        &mut self,
        audience: Option<DID>,
        subject: DID,
        ability: T,
        metadata: BTreeMap<String, Ipld>,
        cause: Option<Cid>,
        expiration: Option<Timestamp>,
        issued_at: Option<Timestamp>,
        now: SystemTime,
        varsig_header: V,
    ) -> Result<
        Invocation<T, DID, V, Enc>,
        InvokeError<
            D::DelegationStoreError,
            ParseAbilityError<()>, // FIXME argserror
        >,
    > {
        let proofs = self
            .delegation_store
            .get_chain(self.did, &Some(subject.clone()), vec![], now)
            .map_err(InvokeError::DelegationStoreError)?
            .map(|chain| chain.map(|(cid, _)| cid).into())
            .unwrap_or(vec![]);

        let mut seed = vec![];

        let payload = Payload {
            issuer: self.did.clone(),
            subject,
            audience,
            ability,
            proofs,
            metadata,
            nonce: Nonce::generate_12(&mut seed),
            cause,
            expiration,
            issued_at,
        };

        Ok(Invocation::try_sign(self.signer, varsig_header, payload)
            .map_err(InvokeError::SignError)?)
    }

    pub fn invoke_promise(
        &mut self,
        audience: Option<&DID>,
        subject: DID,
        ability: T::Promised,
        metadata: BTreeMap<String, Ipld>,
        cause: Option<Cid>,
        expiration: Option<Timestamp>,
        issued_at: Option<Timestamp>,
        now: SystemTime,
        varsig_header: V,
    ) -> Result<
        Invocation<T::Promised, DID, V, Enc>,
        InvokeError<
            D::DelegationStoreError,
            ParseAbilityError<()>, // FIXME errs
        >,
    > {
        let proofs = self
            .delegation_store
            .get_chain(self.did, &Some(subject.clone()), vec![], now)
            .map_err(InvokeError::DelegationStoreError)?
            .map(|chain| chain.map(|(cid, _)| cid).into())
            .unwrap_or(vec![]);

        let mut seed = vec![];

        let payload = Payload {
            issuer: self.did.clone(),
            subject,
            audience: audience.cloned(),
            ability,
            proofs,
            metadata,
            nonce: Nonce::generate_12(&mut seed),
            cause,
            expiration,
            issued_at,
        };

        Ok(Invocation::try_sign(self.signer, varsig_header, payload)
            .map_err(InvokeError::SignError)?)
    }

    pub fn receive(
        &mut self,
        promised: Invocation<T::Promised, DID, V, Enc>,
        now: &SystemTime,
    ) -> Result<
        Recipient<Payload<T, DID>>,
        ReceiveError<T, P, DID, D::DelegationStoreError, S, V, Enc>,
    >
    where
        Enc: From<u32> + Into<u32>,
        arguments::Named<Ipld>: From<T>,
        Invocation<T::Promised, DID, V, Enc>: Clone,
        <P as promise::Store<T, DID>>::PromiseStoreError: fmt::Debug,
        signature::Envelope<Payload<T::Promised, DID>, DID, V, Enc>: Clone,
        <S as Store<<T as Resolvable>::Promised, DID, V, Enc>>::InvocationStoreError: fmt::Debug,
    {
        let cid: Cid = promised.cid().map_err(ReceiveError::EncodingError)?;
        let _ = promised
            .validate_signature()
            .map_err(ReceiveError::SigVerifyError)?;

        self.invocation_store
            .put(cid.clone(), promised.clone())
            .map_err(ReceiveError::InvocationStoreError)?;

        let resolved_ability: T = match Resolvable::try_resolve(promised.ability().clone()) {
            Ok(resolved) => resolved,
            Err(cant_resolve) => {
                let waiting_on: BTreeSet<Cid> = T::get_all_pending(cant_resolve.promised);

                self.unresolved_promise_index
                    .put_waiting(
                        promised.cid()?,
                        waiting_on.into_iter().collect::<Vec<Cid>>(),
                    )
                    .map_err(ReceiveError::PromiseStoreError)?;

                return Ok(Recipient::Unresolved(cid));
            }
        };

        let proof_payloads = self
            .delegation_store
            .get_many(&promised.proofs())
            .map_err(ReceiveError::DelegationStoreError)?
            .into_iter()
            .map(|d| d.payload())
            .collect();

        let resolved_payload = promised.payload().clone().map_ability(|_| resolved_ability);

        let _ = &resolved_payload
            .check(proof_payloads, now)
            .map_err(ReceiveError::ValidationError)?;

        if promised.audience() != &Some(self.did.clone()) {
            return Ok(Recipient::Other(resolved_payload));
        }

        Ok(Recipient::You(resolved_payload))
    }

    pub fn revoke(
        &mut self,
        subject: DID,
        cause: Option<Cid>,
        cid: Cid,
        now: Timestamp,
        varsig_header: V,
        // FIXME return type
    ) -> Result<Invocation<T, DID, V, Enc>, ()>
    where
        T: From<Revoke>,
    {
        let ability: T = Revoke { ucan: cid.clone() }.into();
        let proofs = if &subject == self.did {
            vec![]
        } else {
            self.delegation_store
                .get_chain(&subject, &Some(self.did.clone()), vec![], now.into())
                .map_err(|_| ())?
                .map(|chain| chain.map(|(index_cid, _)| index_cid).into())
                .unwrap_or(vec![])
        };

        let payload = Payload {
            issuer: self.did.clone(),
            subject: self.did.clone(),
            audience: Some(self.did.clone()),
            ability,
            proofs,
            cause,
            metadata: BTreeMap::new(),
            nonce: Nonce::generate_12(&mut vec![]),
            expiration: None,
            issued_at: None,
        };

        let invocation =
            Invocation::try_sign(self.signer, varsig_header, payload).map_err(|_| ())?;

        self.delegation_store.revoke(cid).map_err(|_| ())?;
        Ok(invocation)
    }
}

#[derive(Debug)]
pub enum Recipient<T> {
    // FIXME change to status
    You(T),
    Other(T),
    Unresolved(Cid),
}

#[derive(Debug, Error)]
pub enum ReceiveError<
    T: Resolvable,
    P: promise::Store<T, DID>,
    DID: Did,
    D,
    S: Store<T::Promised, DID, V, Enc>,
    V: varsig::Header<Enc>,
    Enc: Codec + From<u32> + Into<u32>,
> where
    <P as promise::Store<T, DID>>::PromiseStoreError: fmt::Debug,
    <S as Store<<T as Resolvable>::Promised, DID, V, Enc>>::InvocationStoreError: fmt::Debug,
{
    #[error("encoding error: {0}")]
    EncodingError(#[from] libipld_core::error::Error),

    #[error("signature verification error: {0}")]
    SigVerifyError(#[from] signature::ValidateError),

    #[error("invocation store error: {0}")]
    InvocationStoreError(
        #[source] <S as Store<<T as Resolvable>::Promised, DID, V, Enc>>::InvocationStoreError,
    ),

    #[error("promise store error: {0}")]
    PromiseStoreError(#[source] <P as promise::Store<T, DID>>::PromiseStoreError),

    #[error("delegation store error: {0}")]
    DelegationStoreError(#[source] D),

    #[error("delegation validation error: {0}")]
    ValidationError(#[source] ValidationError),
}

#[derive(Debug, Error)]
pub enum InvokeError<D, ArgsErr> {
    #[error("delegation store error: {0}")]
    DelegationStoreError(#[source] D),

    #[error("promise store error: {0}")]
    SignError(#[source] signature::SignError),

    #[error("promise store error: {0}")]
    PromiseResolveError(#[source] ArgsErr),
}
