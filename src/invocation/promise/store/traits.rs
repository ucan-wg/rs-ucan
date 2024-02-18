use crate::{did::Did, invocation::promise::Resolvable};
use libipld_core::cid::Cid;
use std::collections::BTreeSet;

pub trait Store<T: Resolvable, DID: Did> {
    type PromiseIndexError;

    // NOTE put_waiting
    fn put(&mut self, waiting_on: Vec<Cid>, invocation: Cid)
        -> Result<(), Self::PromiseIndexError>;

    // NOTE get waiting
    fn get(&self, waiting_on: &mut Vec<Cid>) -> Result<BTreeSet<Cid>, Self::PromiseIndexError>;
}
