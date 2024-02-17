use super::Store;
use crate::{
    did::Did,
    receipt::{Receipt, Responds},
    task,
};
use libipld_core::ipld::Ipld;
use std::{collections::BTreeMap, convert::Infallible, fmt};

/// An in-memory [`receipt::Store`][crate::receipt::Store].
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStore<T: Responds, DID: Did>
where
    T::Success: fmt::Debug + Clone + PartialEq,
{
    store: BTreeMap<task::Id, Receipt<T, DID>>,
}

impl<T: Responds, DID: Did> Store<T, DID> for MemoryStore<T, DID>
where
    <T as Responds>::Success: TryFrom<Ipld> + Into<Ipld> + Clone + fmt::Debug + PartialEq,
{
    type Error = Infallible;

    fn get(&self, id: &task::Id) -> Result<Option<&Receipt<T, DID>>, Self::Error> {
        Ok(self.store.get(id))
    }

    fn put(&mut self, id: task::Id, receipt: Receipt<T, DID>) -> Result<(), Self::Error> {
        self.store.insert(id, receipt);
        Ok(())
    }
}
