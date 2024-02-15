use super::{payload::Payload, store::Store, Invocation, Resolvable};
use crate::{
    ability::ucan,
    delegation,
    delegation::{condition::Condition, Delegable},
    did::Did,
    nonce::Nonce,
    signature::Verifiable,
    time::JsTime,
};
use libipld_core::{cid::Cid, ipld::Ipld};
use std::{collections::BTreeMap, marker::PhantomData};
use thiserror::Error;
use web_time::SystemTime;

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

    pub delegation_store: &'a D,
    pub invocation_store: &'a mut S,
    pub revocation_store: &'a mut S, // FIXME just a BTRee Set pointing into invocatuin store?

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
{
    pub fn new(
        did: &'a DID,
        signer: &'a <DID as Did>::Signer,
        invocation_store: &'a mut S,
        delegation_store: &'a mut D,
        revocation_store: &'a mut D,
        unresolved_promise_store: &'a mut P,
        resolved_promise_store: &'a mut P,
    ) -> Self {
        Self {
            did,
            invocation_store,
            delegation_store,
            revocation_store,
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
        ability: T::Promised, // FIXME Resolved needs Into<Builder>
        metadata: BTreeMap<String, Ipld>,
        cause: Option<Cid>,
        expiration: Option<JsTime>,
        not_before: Option<JsTime>,
        now: &SystemTime,
        // FIXME err type
    ) -> Result<Invocation<T, DID>, ()> {
        let proofs = self
            .delegation_store
            .get_chain(self.did, subject, &ability.into(), vec![], now)
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
            nonce: Nonce::generate_16(&mut seed),
            cause,
            expiration: expiration.map(Into::into),
            not_before: not_before.map(Into::into),
        };

        let invocation = Invocation::try_sign(self.signer, &payload).map_err(|_| ())?;
        let cid = Cid::from(invocation);
        Ok(invocation)
    }

    // FIXME err = ()
    // FIXME move to revocation agent wit own traits?
    // pub fn revoke(
    //     &mut self,
    //     subject: &DID,
    //     cause: Option<Cid>,
    //     cid: Cid,
    //     now: JsTime,
    // ) -> Result<(), ()> {
    //     let ability = ucan::revoke::Ready { ucan: cid };
    //     let proofs = if subject == self.did {
    //         vec![]
    //     } else {
    //         self.delegation_store
    //             .get_chain(
    //                 subject,
    //                 self.did,
    //                 &ability.into(),
    //                 vec![],
    //                 &SystemTime::now(),
    //             )
    //             .map_err(|_| ())?
    //             .map(|chain| chain.map(|(cid, _)| *cid).into())
    //             .unwrap_or(vec![])
    //     };

    //     let payload = Payload {
    //         issuer: self.did.clone(),
    //         subject: self.did.clone(),
    //         audience: Some(self.did.clone()),
    //         ability,
    //         proofs,
    //         cause: None,
    //         metadata: BTreeMap::new(),
    //         nonce: Nonce::generate_16(&mut vec![]),
    //         expiration: None,
    //         not_before: None,
    //     };

    //     let invocation = Invocation::try_sign(self.signer, &payload).map_err(|_| ())?;

    //     self.invocation_store.revoke(&cid)?;
    // }

    pub fn receive(
        &self,
        invocation: Invocation<T::Promised, DID>,
        now: SystemTime,
        // FIXME return type
    ) -> Result<Recipient<T::Promised>, ()> {
        // FIXME needs varsig header
        let cid = Cid::from(invocation);

        invocation
            .verifier()
            .verify(&cid.to_bytes(), &invocation.signature.to_bytes())
            .map_err(|_| ())?;

        let payload: Payload<T::Promised, DID> = invocation.payload;
        let resolved_payload = match payload.ability.try_resolve() {
            Ok(resolved_payload) => {
                // NOTE Already resolved when it came over the wire
                let resolved_invocation = invocation.map_ability(|_| resolved_payload);
                self.store.put(cid, resolved_invocation).map_err(|_| ())?;
                resolved_payload
            }
            Err(_) => {
                // FIXME check if any of the unresolved promises are in the store
                self.promised_store.put(cid, invocation).map_err(|_| ())?;
                todo!() // return Ok(Recipient::Other(promised)); // FIXME
            }
        };

        let proof_payloads = self
            .delegation_store
            .get_many(&invocation.payload.proofs)
            .map(|d| d.payload);

        resolved_payload
            .into()
            .check(&proof_payloads, now)
            .map_err(|_| ())?;

        if invocation.payload.audience != Some(*self.did) {
            return Ok(Recipient::Other(invocation));
        }

        Ok(Recipient::You(invocation))
    }
}

pub enum Recipient<T> {
    You(T),
    Other(T),
}
