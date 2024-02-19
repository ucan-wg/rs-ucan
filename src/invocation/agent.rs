use super::{payload::Payload, promise::Resolvable, store::Store, Invocation};
use crate::{
    ability::{arguments, ucan},
    crypto::{varsig, Nonce},
    delegation,
    delegation::{condition::Condition, Delegable},
    did::{Did, Verifiable},
    invocation::promise,
    proof::{checkable::Checkable, prove::Prove},
    time::Timestamp,
};
use libipld_cbor::DagCborCodec;
use libipld_core::{
    cid::{Cid, CidGeneric},
    codec::{Codec, Encode},
    ipld::Ipld,
    multihash::{Code, MultihashGeneric},
};
use std::{collections::BTreeMap, fmt, marker::PhantomData};
use thiserror::Error;
use web_time::SystemTime;

#[derive(Debug)]
pub struct Agent<
    'a,
    T: Resolvable + Delegable,
    C: Condition,
    DID: Did,
    S: Store<T, DID, V, Enc>,
    P: promise::Store<T, DID>,
    D: delegation::store::Store<T::Builder, C, DID, V, Enc>,
    V: varsig::Header<Enc>,
    Enc: Codec + Into<u32> + TryFrom<u32>,
> {
    pub did: &'a DID,

    pub delegation_store: &'a mut D,
    pub invocation_store: &'a mut S,
    pub unresolved_promise_store: &'a mut P,
    pub resolved_promise_store: &'a mut P,

    signer: &'a <DID as Did>::Signer,
    marker: PhantomData<(T, C, V, Enc)>,
}

impl<
        'a,
        T: Resolvable + Delegable + Clone,
        C: Condition,
        DID: Did + Clone,
        S: Store<T, DID, V, Enc>,
        P: promise::Store<T, DID>,
        D: delegation::store::Store<T::Builder, C, DID, V, Enc>,
        V: varsig::Header<Enc>,
        Enc: Codec + Into<u32> + TryFrom<u32>,
    > Agent<'a, T, C, DID, S, P, D, V, Enc>
where
    T::Promised: Clone,
    Ipld: Encode<Enc>,
    delegation::Payload<<T::Builder as Checkable>::Hierarchy, C, DID>: Clone,
{
    pub fn new(
        did: &'a DID,
        signer: &'a <DID as Did>::Signer,
        invocation_store: &'a mut S,
        delegation_store: &'a mut D,
        unresolved_promise_store: &'a mut P,
        resolved_promise_store: &'a mut P,
    ) -> Self {
        Self {
            did,
            invocation_store,
            delegation_store,
            unresolved_promise_store,
            resolved_promise_store,
            signer,
            marker: PhantomData,
        }
    }

    pub fn invoke(
        &mut self,
        audience: Option<&DID>,
        subject: &DID,
        ability: T::Promised, // FIXME give them an enum for promised or not probs doens't matter?
        metadata: BTreeMap<String, Ipld>,
        cause: Option<Cid>,
        expiration: Option<Timestamp>,
        issued_at: Option<Timestamp>,
        now: SystemTime,
        varsig_header: V,
        // FIXME err type
    ) -> Result<Invocation<T::Promised, DID, V, Enc>, ()> {
        let proofs = self
            .delegation_store
            .get_chain(self.did, subject, &ability.clone().into(), vec![], now)
            .map_err(|_| ())?
            .map(|chain| chain.map(|(cid, _)| cid).into())
            .unwrap_or(vec![]);

        let mut seed = vec![];

        let payload = Payload {
            issuer: self.did.clone(),
            subject: subject.clone(),
            audience: audience.cloned(),
            ability,
            proofs,
            metadata,
            nonce: Nonce::generate_12(&mut seed),
            cause,
            expiration,
            issued_at,
        };

        Ok(Invocation::try_sign(self.signer, varsig_header, payload).map_err(|_| ())?)
    }

    pub fn receive(
        &mut self,
        promised: Invocation<T::Promised, DID, V, Enc>,
        now: &SystemTime,
    ) -> Result<Recipient<Payload<T, DID>>, ReceiveError<T, P, DID, C, D::DelegationStoreError>>
    where
        C: fmt::Debug + Clone,
        <T::Builder as Checkable>::Hierarchy: Clone + Into<arguments::Named<Ipld>>,
        T::Builder: Clone + Checkable + Prove + Into<arguments::Named<Ipld>>,
        Invocation<T::Promised, DID, V, Enc>: Clone,
        <<<T as Delegable>::Builder as Checkable>::Hierarchy as Prove>::Error: fmt::Debug,
        <P as promise::Store<T, DID>>::PromiseStoreError: fmt::Debug,
    {
        let mut buffer = vec![];
        Ipld::from(promised.clone())
            .encode(*promised.varsig_header().codec(), &mut buffer)
            .map_err(ReceiveError::EncodingError)?;

        let cid: Cid = CidGeneric::new_v1(
            DagCborCodec.into(),
            MultihashGeneric::wrap(Code::Sha2_256.into(), buffer.as_slice())
                .map_err(ReceiveError::MultihashError)?,
        );

        let mut encoded = vec![];
        Ipld::from(promised.payload().clone())
            .encode(*promised.0.varsig_header.codec(), &mut encoded)
            .map_err(ReceiveError::EncodingError)?;

        promised
            .verifier()
            .verify(&encoded, &promised.signature())
            .map_err(ReceiveError::SigVerifyError)?;

        let resolved_ability: T = match Resolvable::try_resolve(promised.ability().clone()) {
            Ok(resolved) => resolved,
            Err(_) => {
                // FIXME check if any of the unresolved promises are in the store
                // FIXME check if it's actually unresolved

                // self.invocation_store
                //     .put(cid.clone(), promised.clone())
                //     .map_err(ReceiveError::PromiseStoreError)?;

                self.unresolved_promise_store
                    .put(cid, todo!()) // cid for promised)
                    .map_err(ReceiveError::PromiseStoreError)?;

                todo!()
                // return Ok(Recipient::Other(promised)); // FIXME
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

        delegation::Payload::<T::Builder, C, DID>::from(resolved_payload.clone())
            .check(proof_payloads, now)
            .map_err(ReceiveError::DelegationValidationError)?;

        if promised.audience() != &Some(self.did.clone()) {
            return Ok(Recipient::Other(resolved_payload));
        }

        Ok(Recipient::You(resolved_payload))
    }

    pub fn revoke(
        &mut self,
        subject: &DID,
        cause: Option<Cid>,
        cid: Cid,
        now: Timestamp,
        varsig_header: V,
        // FIXME return type
    ) -> Result<Invocation<T, DID, V, Enc>, ()>
    where
        T: From<ucan::revoke::Ready>,
    {
        let ability: T = ucan::revoke::Ready { ucan: cid.clone() }.into();
        let proofs = if subject == self.did {
            vec![]
        } else {
            self.delegation_store
                .get_chain(
                    subject,
                    self.did,
                    &ability.clone().into(),
                    vec![],
                    now.into(),
                )
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
    You(T),
    Other(T),
}

#[derive(Debug, Error)]
pub enum ReceiveError<T: Resolvable, P: promise::Store<T, DID>, DID: Did, C: fmt::Debug, D>
where
    delegation::ValidationError<
        <<<T as Delegable>::Builder as Checkable>::Hierarchy as Prove>::Error,
        C,
    >: fmt::Debug,
    <P as promise::Store<T, DID>>::PromiseStoreError: fmt::Debug,
{
    #[error("encoding error: {0}")]
    EncodingError(#[from] libipld_core::error::Error),

    #[error("multihash error: {0}")]
    MultihashError(#[from] libipld_core::multihash::Error),

    #[error("signature verification error: {0}")]
    SigVerifyError(#[from] signature::Error),

    #[error("promise store error: {0}")]
    PromiseStoreError(#[source] <P as promise::Store<T, DID>>::PromiseStoreError),

    #[error("delegation store error: {0}")]
    DelegationStoreError(#[source] D),

    #[error("delegation validation error: {0}")]
    DelegationValidationError(
        #[source]
        delegation::ValidationError<
            <<<T as Delegable>::Builder as Checkable>::Hierarchy as Prove>::Error,
            C,
        >,
    ),
}
