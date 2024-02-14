use super::{payload::Payload, store::Store, Invocation, Resolvable};
use crate::{
    delegation,
    delegation::{condition::Condition, Delegable, Delegation},
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

    pub store: &'a mut S,
    pub promised_store: &'a mut P,
    pub delegation_store: &'a D,

    signer: &'a <DID as Did>::Signer,
    marker: PhantomData<(T, C)>,
}

impl<
        'a,
        T: Resolvable + Delegable + Clone,
        C: Condition,
        DID: Did + ToString + Clone,
        S: Store<T, DID>,
        P: Store<T::Promised, DID>,
        D: delegation::store::Store<T::Builder, C, DID>,
    > Agent<'a, T, C, DID, S, P, D>
{
    pub fn new(
        did: &'a DID,
        signer: &'a <DID as Did>::Signer,
        store: &'a mut S,
        promised_store: &'a mut P,
        delegation_store: &'a mut D,
    ) -> Self {
        Self {
            did,
            store,
            promised_store,
            delegation_store,
            signer,
            marker: PhantomData,
        }
    }

    pub fn invoke(
        &mut self,
        audience: Option<&DID>,
        subject: &DID,
        ability: T::Promised,
        metadata: BTreeMap<String, Ipld>,
        cause: Option<Cid>,
        expiration: JsTime,
        not_before: Option<JsTime>,
        // FIXME err type
    ) -> Result<Invocation<T, DID>, ()> {
        let proofs = self
            .delegation_store
            .get_chain(
                audience,
                subject,
                ability.into(),
                vec![],
                &SystemTime::now(),
            )
            .map_err(|_| ())?
            .map(|chain| chain.map(|(cid, _)| *cid).into())
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
            expiration,
            not_before,
        };

        let invocation = Invocation::try_sign(self.signer, &payload).map_err(|_| ())?;
        let cid: Cid = invocation.into();
        self.store.put(cid, invocation);
        Ok(invocation)
    }

    // FIXME err = ()
    pub fn revoke(&mut self, cid: &Cid) -> Result<(), ()> {
        todo!(); // FIXME create a revoke invocation
        self.store.revoke(&cid)
    }

    pub fn receive(
        &self,
        invocation: Invocation<T::Promised, DID>,
        proofs: BTreeMap<Cid, Delegation<T::Builder, C, DID>>,
        // FIXME return type
    ) -> Result<Recipient<T::Promised>, ()> {
        // FIXME needs varsig header
        let cid = Cid::from(invocation);

        invocation
            .verifier()
            .verify(&cid.to_bytes(), &invocation.signature.to_bytes())
            .map_err(|_| ())?;

        // FIXME pull delegations out of the store and check them

        match Resolvable::try_resolve(&invocation.payload) {
            Ok(resolved) => {
                // FIXME promised store
                self.store.put(cid, resolved).map_err(|_| ())?;
            }
            Err(unresolved) => self.promised_store.put(cid, unresolved).map_err(|_| ())?,
        }

        // FIXME
        // FIXME promised store
        self.store.put(cid, invocation).map_err(|_| ())?;

        for (cid, deleg) in proofs {
            self.delegation_store.insert(cid, deleg).map_err(|_| ())?;
        }

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
