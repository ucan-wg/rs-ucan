use super::Store;
use crate::{did::Did, invocation::promise::Resolvable};
use libipld_core::cid::Cid;
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::Infallible,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStore {
    pub index: BTreeMap<Cid, BTreeSet<Cid>>,
}

impl<T: Resolvable, DID: Did> Store<T, DID> for MemoryStore {
    type PromiseStoreError = Infallible;

    fn put(
        &mut self,
        invocation: Cid,
        waiting_on: Vec<Cid>,
    ) -> Result<(), Self::PromiseStoreError> {
        self.index
            .insert(invocation, BTreeSet::from_iter(waiting_on));

        Ok(())
    }

    fn get(&self, waiting_on: &mut Vec<Cid>) -> Result<BTreeSet<Cid>, Self::PromiseStoreError> {
        Ok(match waiting_on.pop() {
            None => BTreeSet::new(),
            Some(first) => waiting_on
                .iter()
                .try_fold(BTreeSet::from_iter([first]), |acc, cid| {
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
