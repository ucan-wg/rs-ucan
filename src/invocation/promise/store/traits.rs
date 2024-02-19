use crate::{did::Did, invocation::promise::Resolvable};
use libipld_core::cid::Cid;
use std::collections::BTreeSet;

pub trait Store<T: Resolvable, DID: Did> {
    type PromiseStoreError;

    // NOTE put_waiting
    fn put(&mut self, invocation: Cid, waiting_on: Vec<Cid>)
        -> Result<(), Self::PromiseStoreError>;

    // NOTE get waiting
    fn get(&self, waiting_on: &mut Vec<Cid>) -> Result<BTreeSet<Cid>, Self::PromiseStoreError>;
}
