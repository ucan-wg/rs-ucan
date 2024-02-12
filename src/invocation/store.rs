use super::Invocation;
use libipld_core::cid::Cid;
use std::collections::BTreeMap;
use thiserror::Error;

pub trait Store<T> {
    type Error;

    fn get(&self, cid: &Cid) -> Result<&Invocation<T>, Self::Error>;

    fn put(&mut self, cid: Cid, invocation: Invocation<T>) -> Result<(), Self::Error>;

    fn has(&self, cid: &Cid) -> Result<bool, Self::Error> {
        Ok(self.get(cid).is_ok())
    }
}

pub struct MemoryStore<T> {
    store: BTreeMap<Cid, Invocation<T>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
#[error("Delegation not found")]
pub struct NotFound;

impl<T> Store<T> for MemoryStore<T> {
    type Error = NotFound;

    fn get(&self, cid: &Cid) -> Result<&Invocation<T>, Self::Error> {
        self.store.get(cid).ok_or(NotFound)
    }

    fn put(&mut self, cid: Cid, invocation: Invocation<T>) -> Result<(), Self::Error> {
        self.store.insert(cid, invocation);
        Ok(())
    }
}
