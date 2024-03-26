use libipld_core::cid::Cid;
use std::collections::BTreeSet;

pub trait Store {
    type PromiseStoreError;

    fn put_waiting(
        &mut self,
        invocation: Cid,
        waiting_on: Vec<Cid>,
    ) -> Result<(), Self::PromiseStoreError>;

    fn get_waiting(
        &self,
        waiting_on: &mut Vec<Cid>,
    ) -> Result<BTreeSet<Cid>, Self::PromiseStoreError>;
}
