//! Storage for [`Invocation`]s.

use super::Invocation;
use crate::{crypto::varsig, did::Did};
use libipld_core::{cid::Cid, codec::Codec};
use std::{collections::BTreeMap, convert::Infallible};

pub trait Store<T, DID: Did, V: varsig::Header<Enc>, Enc: Codec + Into<u32> + TryFrom<u32>> {
    type InvocationStoreError;

    fn get(
        &self,
        cid: Cid,
    ) -> Result<Option<&Invocation<T, DID, V, Enc>>, Self::InvocationStoreError>;

    fn put(
        &mut self,
        cid: Cid,
        invocation: Invocation<T, DID, V, Enc>,
    ) -> Result<(), Self::InvocationStoreError>;

    fn has(&self, cid: Cid) -> Result<bool, Self::InvocationStoreError> {
        Ok(self.get(cid).is_ok())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStore<T, DID: Did, V: varsig::Header<Enc>, Enc: Codec + Into<u32> + TryFrom<u32>> {
    store: BTreeMap<Cid, Invocation<T, DID, V, Enc>>,
}

impl<T, DID: Did, V: varsig::Header<Enc>, Enc: Codec + Into<u32> + TryFrom<u32>>
    Store<T, DID, V, Enc> for MemoryStore<T, DID, V, Enc>
{
    type InvocationStoreError = Infallible;

    fn get(
        &self,
        cid: Cid,
    ) -> Result<Option<&Invocation<T, DID, V, Enc>>, Self::InvocationStoreError> {
        Ok(self.store.get(&cid))
    }

    fn put(
        &mut self,
        cid: Cid,
        invocation: Invocation<T, DID, V, Enc>,
    ) -> Result<(), Self::InvocationStoreError> {
        self.store.insert(cid, invocation);
        Ok(())
    }
}
