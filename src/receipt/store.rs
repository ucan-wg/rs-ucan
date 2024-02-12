use super::{Receipt, Responds};
use crate::task;
use libipld_core::ipld::Ipld;
use std::{collections::BTreeMap, fmt};
use thiserror::Error;

/// A store for [`Receipt`]s indexed by their [`task::Id`]s.
pub trait Store<T: Responds> {
    /// The error type representing all the ways a store operation can fail.
    type Error;

    /// Retrieve a [`Receipt`] by its [`task::Id`].
    fn get<'a>(&self, id: &task::Id) -> Result<&Receipt<T>, Self::Error>
    where
        <T as Responds>::Success: TryFrom<Ipld>;

    /// Store a [`Receipt`] by its [`task::Id`].
    fn put(&mut self, id: task::Id, receipt: Receipt<T>) -> Result<(), Self::Error>
    where
        <T as Responds>::Success: Into<Ipld>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStore<T: Responds>
where
    T::Success: fmt::Debug + Clone + PartialEq,
{
    store: BTreeMap<task::Id, Receipt<T>>,
}

// FIXME extract
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
#[error("Delegation not found")]
pub struct NotFound;

impl<T: Responds> Store<T> for MemoryStore<T>
where
    <T as Responds>::Success: TryFrom<Ipld> + Into<Ipld> + Clone + fmt::Debug + PartialEq,
{
    type Error = NotFound;

    fn get(&self, id: &task::Id) -> Result<&Receipt<T>, Self::Error> {
        self.store.get(id).ok_or(NotFound)
    }

    fn put(&mut self, id: task::Id, receipt: Receipt<T>) -> Result<(), Self::Error> {
        self.store.insert(id, receipt);
        Ok(())
    }
}
