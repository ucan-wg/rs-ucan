use super::Invocation;
use crate::{did::Did, invocation::Resolvable};
use libipld_core::{cid::Cid, link::Link};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::ControlFlow,
};
use thiserror::Error;

pub trait Store<T, DID: Did> {
    type Error;

    fn get(&self, cid: Cid) -> Result<&Invocation<T, DID>, Self::Error>;

    fn put(&mut self, cid: Cid, invocation: Invocation<T, DID>) -> Result<(), Self::Error>;

    fn has(&self, cid: Cid) -> Result<bool, Self::Error> {
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

    fn get(&self, cid: Cid) -> Result<&Invocation<T, DID>, Self::Error> {
        self.store.get(&cid).ok_or(NotFound)
    }

    fn put(&mut self, cid: Cid, invocation: Invocation<T, DID>) -> Result<(), Self::Error> {
        self.store.insert(cid, invocation);
        Ok(())
    }
}

////////

pub trait PromiseIndex<T: Resolvable, DID: Did> {
    type PromiseIndexError;

    fn put_waiting(
        &mut self,
        waiting_on: Vec<Cid>,
        invocation: Cid,
    ) -> Result<(), Self::PromiseIndexError>;

    fn get_waiting(
        &self,
        waiting_on: &mut Vec<Cid>,
    ) -> Result<BTreeSet<Cid>, Self::PromiseIndexError>;
}

pub struct MemoryPromiseIndex {
    pub index: BTreeMap<Cid, BTreeSet<Cid>>,
}

impl<T: Resolvable, DID: Did> PromiseIndex<T, DID> for MemoryPromiseIndex {
    type PromiseIndexError = NotFound;

    fn put_waiting(
        &mut self,
        waiting_on: Vec<Cid>,
        invocation: Cid,
    ) -> Result<(), Self::PromiseIndexError> {
        self.index
            .insert(invocation, BTreeSet::from_iter(waiting_on));

        Ok(())
    }

    fn get_waiting(
        &self,
        waiting_on: &mut Vec<Cid>,
    ) -> Result<BTreeSet<Cid>, Self::PromiseIndexError> {
        Ok(match waiting_on.pop() {
            None => BTreeSet::new(),
            Some(first) => waiting_on
                .iter()
                .try_fold(BTreeSet::from_iter([first]), |mut acc, cid| {
                    let next = self.index.get(cid).ok_or(())?;

                    let reduced: BTreeSet<Cid> = acc.intersection(&next).cloned().collect();
                    if reduced.is_empty() {
                        return Err(());
                    }

                    Ok(reduced)
                })
                .unwrap_or_default(),
        })
    }
}
