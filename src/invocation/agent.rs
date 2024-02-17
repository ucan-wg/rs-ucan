use super::{payload::Payload, store::Store, Invocation, Resolvable};
use crate::{
    ability::{arguments, ucan},
    delegation,
    delegation::{condition::Condition, Delegable},
    did::Did,
    nonce::Nonce,
    proof::{checkable::Checkable, prove::Prove},
    signature::{Signature, Verifiable},
    time::JsTime,
};
use libipld_cbor::DagCborCodec;
use libipld_core::{
    cid::{Cid, CidGeneric},
    codec::Encode,
    ipld::Ipld,
    multihash::{Code, MultihashGeneric},
};
use std::{collections::BTreeMap, marker::PhantomData};
use web_time::SystemTime;

#[derive(Debug)]
pub struct Agent<
    'a,
    T: Resolvable + Delegable,
    C: Condition,
    DID: Did,
    S: Store<T, DID>,
    P: Store<T::Promised, DID>,
    D: delegation::store::Store<T::Builder, C, DID>,
> {
    pub did: &'a DID,

    pub delegation_store: &'a mut D,
    pub invocation_store: &'a mut S,
    pub unresolved_promise_store: &'a mut P,
    pub resolved_promise_store: &'a mut P,

    signer: &'a <DID as Did>::Signer,
    marker: PhantomData<(T, C)>,
}

impl<
        'a,
        T: Resolvable + Delegable + Clone,
        C: Condition,
        DID: Did + Clone,
        S: Store<T, DID>,
        P: Store<T::Promised, DID>,
        D: delegation::store::Store<T::Builder, C, DID>,
    > Agent<'a, T, C, DID, S, P, D>
where
    T::Promised: Clone,
    // Payload<<T::Builder as Checkable>::Hierarchy, DID>: Clone, // FIXME
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
        expiration: Option<JsTime>,
        not_before: Option<JsTime>,
        now: &SystemTime,
        // FIXME err type
    ) -> Result<Invocation<T::Promised, DID>, ()> {
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
            expiration: expiration.map(Into::into),
            not_before: not_before.map(Into::into),
        };

        Ok(Invocation::try_sign(self.signer, payload).map_err(|_| ())?)
    }

    pub fn receive(
        &mut self,
        promised: Invocation<T::Promised, DID>,
        now: &SystemTime,
        // FIXME return type
    ) -> Result<Recipient<Payload<T, DID>>, ()>
    where
        <T::Builder as Checkable>::Hierarchy: Clone + Into<arguments::Named<Ipld>>,
        T::Builder: Clone + Checkable + Prove + Into<arguments::Named<Ipld>>,
    {
        // FIXME needs varsig header
        let mut buffer = vec![];
        Ipld::from(promised.clone())
            .encode(DagCborCodec, &mut buffer)
            .expect("FIXME not dag-cbor? DagCborCodec to encode any arbitrary `Ipld`");

        let cid: Cid = CidGeneric::new_v1(
            DagCborCodec.into(),
            MultihashGeneric::wrap(Code::Sha2_256.into(), buffer.as_slice())
                .map_err(|_| ()) // FIXME
                .expect("FIXME expect signing to work..."),
        );

        let mut encoded = vec![];
        promised
            .payload
            // FIXME use the varsig headre to get the codec
            .encode(DagCborCodec, &mut encoded)
            .expect("FIXME");

        promised
            .verifier()
            .verify(
                &encoded,
                &match promised.signature {
                    Signature::Solo(ref sig) => sig.clone(),
                },
            )
            .map_err(|_| ())?;

        let resolved_ability: T = match Resolvable::try_resolve(promised.payload.ability.clone()) {
            Ok(resolved) => resolved,
            Err(_) => {
                // FIXME check if any of the unresolved promises are in the store
                // FIXME check if it's actually unresolved
                self.unresolved_promise_store
                    .put(cid, promised)
                    .map_err(|_| ())?;

                todo!()
                // return Ok(Recipient::Other(promised)); // FIXME
            }
        };

        let proof_payloads = self
            .delegation_store
            .get_many(&promised.payload.proofs)
            .map_err(|_| ())?
            .into_iter()
            .map(|d| d.payload.clone())
            .collect();

        let resolved_payload = promised.payload.clone().map_ability(|_| resolved_ability);

        delegation::Payload::<T::Builder, C, DID>::from(resolved_payload.clone())
            .check(proof_payloads, now)
            .map_err(|_| ())?;

        if promised.payload.audience != Some(self.did.clone()) {
            return Ok(Recipient::Other(resolved_payload));
        }

        Ok(Recipient::You(resolved_payload))
    }

    pub fn revoke(
        &mut self,
        subject: &DID,
        cause: Option<Cid>,
        cid: Cid,
        now: JsTime,
        // FIXME return type
    ) -> Result<Invocation<T, DID>, ()>
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
                    &now.into(),
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
            not_before: None,
        };

        let invocation = Invocation::try_sign(self.signer, payload).map_err(|_| ())?;

        self.delegation_store.revoke(cid).map_err(|_| ())?;
        Ok(invocation)
    }
}

#[derive(Debug)]
pub enum Recipient<T> {
    You(T),
    Other(T),
}

// impl<T> Agent {
// FIXME err = ()
// FIXME move to revocation agent wit own traits?
// }
