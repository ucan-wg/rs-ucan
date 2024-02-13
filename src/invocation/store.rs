use super::Invocation;
use crate::did::Did;
use libipld_core::cid::Cid;
use std::collections::BTreeMap;
use thiserror::Error;

pub trait Store<T, DID: Did> {
    type Error;

    fn get(&self, cid: &Cid) -> Result<&Invocation<T, DID>, Self::Error>;

    fn put(&mut self, cid: Cid, invocation: Invocation<T, DID>) -> Result<(), Self::Error>;

    fn has(&self, cid: &Cid) -> Result<bool, Self::Error> {
        Ok(self.get(cid).is_ok())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStore<T, DID: Did> {
    store: BTreeMap<Cid, Invocation<T, DID>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
#[error("Delegation not found")]
pub struct NotFound;

impl<T, DID: Did> Store<T, DID> for MemoryStore<T, DID> {
    type Error = NotFound;

    fn get(&self, cid: &Cid) -> Result<&Invocation<T, DID>, Self::Error> {
        self.store.get(cid).ok_or(NotFound)
    }

    fn put(&mut self, cid: Cid, invocation: Invocation<T, DID>) -> Result<(), Self::Error> {
        self.store.insert(cid, invocation);
        Ok(())
    }
}
