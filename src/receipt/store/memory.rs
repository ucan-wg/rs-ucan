use super::Store;
use crate::{
    crypto::varsig,
    did::Did,
    receipt::{Receipt, Responds},
    task,
};
use libipld_core::{codec::Codec, ipld::Ipld};
use std::{collections::BTreeMap, convert::Infallible, fmt};

/// An in-memory [`receipt::Store`][crate::receipt::Store].
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStore<
    T: Responds,
    DID: Did,
    V: varsig::Header<Enc>,
    Enc: Codec + Into<u32> + TryFrom<u32>,
> where
    T::Success: fmt::Debug + Clone + PartialEq,
{
    store: BTreeMap<task::Id, Receipt<T, DID, V, Enc>>,
}

impl<T: Responds, DID: Did, V: varsig::Header<Enc>, Enc: Codec + Into<u32> + TryFrom<u32>>
    Store<T, DID, V, Enc> for MemoryStore<T, DID, V, Enc>
where
    <T as Responds>::Success: TryFrom<Ipld> + Into<Ipld> + Clone + fmt::Debug + PartialEq,
{
    type Error = Infallible;

    fn get(&self, id: &task::Id) -> Result<Option<&Receipt<T, DID, V, Enc>>, Self::Error> {
        Ok(self.store.get(id))
    }

    fn put(&mut self, id: task::Id, receipt: Receipt<T, DID, V, Enc>) -> Result<(), Self::Error> {
        self.store.insert(id, receipt);
        Ok(())
    }
}
