use crate::{
    ability::command::ToCommand,
    delegation::{traits::Condition, Delegatable, Delegation},
    did::Did,
    invocation::Invocation,
    metadata as meta,
    proof::parents::CheckParents,
};

pub struct Agent<S> {
    pub did: Did,
    // pub key: signature::Key,
    pub store: S,
}

impl<S> Agent<S> {
    //  pub fn delegate<T, C, E>(&self, payload: Payload<T, C, E>) -> Delegation<T, C, E> {
    //      let signature = self.key.sign(payload);
    //      signature::Envelope::new(payload, signature)
    //  }

    pub fn invoke<T: Delegatable + CheckParents, C: Condition, E: meta::MultiKeyed>(
        &self,
        delegation: Delegation<T, C, E>,
        proof_chain: Vec<Delegation<T::Parents, C, E>>, // FIXME T must also accept Self and *
    ) -> ()
    where
        T::Parents: Delegatable,
    {
        todo!()
    }

    pub fn try_invoke<A: ToCommand>(&self, ability: A) {
        todo!()
    }

    pub fn revoke<T: Delegatable + CheckParents, C: Condition, E: meta::MultiKeyed>(
        &self,
        delegation: Delegation<T, C, E>,
    ) -> ()
//     where
//         T::Parents: Delegatable,
    {
        todo!()
    }

    pub fn receive_delegation<T: Delegatable + CheckParents, C: Condition, E: meta::MultiKeyed>(
        &self,
        delegation: Delegation<T, C, E>,
    ) -> () {
        todo!()
    }

    pub fn receive_invocation<T, E: meta::MultiKeyed>(&self, invocation: Invocation<T, E>) -> () {
        todo!()
    }

    //  pub fn check(&self, delegation: &Delegation) -> () // FIXME Includes cache
}
