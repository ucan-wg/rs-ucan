//! Storage for [`Invocation`]s.

use super::Invocation;
use crate::did::Did;
use libipld_core::cid::Cid;
use std::{collections::BTreeMap, convert::Infallible};

pub trait Store<T, DID: Did> {
    type Error;

    fn get(&self, cid: Cid) -> Result<Option<&Invocation<T, DID>>, Self::Error>;

    fn put(&mut self, cid: Cid, invocation: Invocation<T, DID>) -> Result<(), Self::Error>;

    fn has(&self, cid: Cid) -> Result<bool, Self::Error> {
        Ok(self.get(cid).is_ok())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStore<T, DID: Did> {
    store: BTreeMap<Cid, Invocation<T, DID>>,
}

impl<T, DID: Did> Store<T, DID> for MemoryStore<T, DID> {
    type Error = Infallible;

    fn get(&self, cid: Cid) -> Result<Option<&Invocation<T, DID>>, Self::Error> {
        Ok(self.store.get(&cid))
    }

    fn put(&mut self, cid: Cid, invocation: Invocation<T, DID>) -> Result<(), Self::Error> {
        self.store.insert(cid, invocation);
        Ok(())
    }
}
