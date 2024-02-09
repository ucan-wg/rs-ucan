use crate::{
    ability::command::ToCommand,
    delegation::{condition::Condition, Delegatable, Delegation},
    did::Did,
    invocation::Invocation,
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

    pub fn invoke<T: Delegatable + CheckParents, C: Condition>(
        &self,
        delegation: Delegation<T, C>,
        proof_chain: Vec<Delegation<T::Parents, C>>, // FIXME T must also accept Self and *
    ) -> ()
    where
        T::Parents: Delegatable,
    {
        todo!()
    }

    pub fn try_invoke<A: ToCommand>(&self, ability: A) {
        todo!()
    }

    pub fn revoke<T: Delegatable + CheckParents, C: Condition>(
        &self,
        delegation: Delegation<T, C>,
    ) -> ()
//     where
//         T::Parents: Delegatable,
    {
        todo!()
    }

    pub fn receive_delegation<T: Delegatable + CheckParents, C: Condition>(
        &self,
        delegation: Delegation<T, C>,
    ) -> () {
        todo!()
    }

    pub fn receive_invocation<T>(&self, invocation: Invocation<T>) -> () {
        todo!()
    }

    //  pub fn check(&self, delegation: &Delegation) -> () // FIXME Includes cache
}
