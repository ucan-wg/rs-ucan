use crate::{
    delegation,
    delegation::{condition::Condition, Delegable, Delegation},
    did::Did,
    invocation,
    nonce::Nonce,
    proof::checkable::Checkable,
    receipt,
    receipt::Responds,
    time::JsTime,
};
use libipld_core::ipld::Ipld;
use nonempty::NonEmpty;
use std::{collections::BTreeMap, marker::PhantomData};
use web_time::SystemTime;

// FIXME move proofs to under delegation?

#[derive(Debug, Clone, PartialEq)]
pub struct Agent<
    T: Delegable + Responds,
    C: Condition,
    S: delegation::store::Store<T::Builder, C>
        + invocation::store::Store<T>
        + receipt::store::Store<T>,
> {
    pub did: Did,
    // pub key: signature::Key,
    pub store: S,
    pub _phantom: PhantomData<(T, C)>,
}

impl<
        T: Delegable + Responds,
        C: Condition + Clone,
        S: delegation::store::Store<T::Builder, C>
            + invocation::store::Store<T>
            + receipt::store::Store<T>,
    > Agent<T, C, S>
{
    fn new(did: Did, store: S) -> Self {
        Self {
            did,
            store,
            _phantom: PhantomData,
        }
    }

    pub fn delegate(
        &self,
        audience: Did,
        subject: Did,
        ability_builder: T::Builder,
        new_conditions: Vec<C>,
        metadata: BTreeMap<String, Ipld>,
        expiration: JsTime,
        not_before: Option<JsTime>,
    ) -> Result<delegation::Delegation<T::Builder, C>, ()> {
        // FIXME check if possible in store first;

        let conditions = if subject == self.did {
            new_conditions
        } else {
            let mut conds = self
                .store
                .get_chain(&self.did, &subject, &ability_builder, SystemTime::now())
                .map_err(|_| ())? // FIXME
                .first()
                .1
                .payload
                .conditions;

            let mut new = new_conditions;
            conds.append(&mut new);
            conds
        };

        let mut salt = self.did.clone().to_string().into_bytes();

        let payload = delegation::Payload {
            issuer: self.did.clone(),
            audience,
            subject,
            ability_builder,
            conditions,
            metadata,
            nonce: Nonce::generate_16(&mut salt),
            expiration: expiration.into(),
            not_before: not_before.map(Into::into),
        };

        Ok(self.sign_delegation(payload))
    }

    pub fn sign_delegation(
        &self,
        payload: delegation::Payload<T::Builder, C>,
    ) -> delegation::Delegation<T::Builder, C> {
        // FIXME check if possible in store first;
        let signature = todo!(); // self.key.sign(payload);
        Delegation { payload, signature }
    }

    pub fn receive_delegation() {}
}

// impl<S> Agent<S> {
// }
//
//
//
//
//
//  pub fn delegate<T, C, E>(&self, payload: Payload<T, C, E>) -> Delegation<T, C, E> {
//      let signature = self.key.sign(payload);
//      signature::Envelope::new(payload, signature)
//  }

// pub fn invoke<T: Delegable + CheckParents, C: Condition>(
//     &self,
//     delegation: Delegation<T, C>,
//     proof_chain: Vec<Delegation<T::Parents, C>>, // FIXME T must also accept Self and *
// ) -> ()
// where
//     T::Parents: Delegable,
// {
//     todo!()
// }

// pub fn try_invoke<A: ToCommand>(&self, ability: A) {
//     todo!()
// }

// pub fn revoke<T: Delegable + CheckParents, C: Condition>(
//     &self,
//     delegation: Delegation<T, C>,
// ) -> ()
//  //    where
//  //        T::Parents: Delegable,
// {
//     todo!()
// }

// pub fn receive_delegation<T: Delegable + CheckParents, C: Condition>(
//     &self,
//     delegation: Delegation<T, C>,
// ) -> () {
//     todo!()
// }

// pub fn receive_invocation<T>(&self, invocation: Invocation<T>) -> () {
//     todo!()
// }

//  pub fn check(&self, delegation: &Delegation) -> () // FIXME Includes cache
