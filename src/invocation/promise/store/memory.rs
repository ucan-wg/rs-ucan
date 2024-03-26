use super::Store;
use libipld_core::cid::Cid;
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::Infallible,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MemoryStore {
    pub index: BTreeMap<Cid, BTreeSet<Cid>>,
}

impl Store for MemoryStore {
    type PromiseStoreError = Infallible;

    fn put_waiting(
        &mut self,
        invocation: Cid,
        waiting_on: Vec<Cid>,
    ) -> Result<(), Self::PromiseStoreError> {
        self.index
            .insert(invocation, BTreeSet::from_iter(waiting_on));

        Ok(())
    }

    fn get_waiting(
        &self,
        waiting_on: &mut Vec<Cid>,
    ) -> Result<BTreeSet<Cid>, Self::PromiseStoreError> {
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
