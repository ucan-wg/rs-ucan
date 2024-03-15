//! Storage for [`Invocation`]s.

use super::Invocation;
use crate::ability;
use crate::{crypto::varsig, did::Did};
use libipld_core::{cid::Cid, codec::Codec};
use std::{collections::BTreeMap, convert::Infallible};

pub trait Store<T, DID: Did, V: varsig::Header<C>, C: Codec + Into<u64> + TryFrom<u64>> {
    type InvocationStoreError;

    fn get(
        &self,
        cid: Cid,
    ) -> Result<Option<&Invocation<T, DID, V, C>>, Self::InvocationStoreError>;

    fn put(
        &mut self,
        cid: Cid,
        invocation: Invocation<T, DID, V, C>,
    ) -> Result<(), Self::InvocationStoreError>;

    fn has(&self, cid: Cid) -> Result<bool, Self::InvocationStoreError> {
        Ok(self.get(cid).is_ok())
    }
}

impl<
        S: Store<T, DID, V, C>,
        T,
        DID: Did,
        V: varsig::Header<C>,
        C: Codec + Into<u64> + TryFrom<u64>,
    > Store<T, DID, V, C> for &mut S
{
    type InvocationStoreError = <S as Store<T, DID, V, C>>::InvocationStoreError;

    fn get(
        &self,
        cid: Cid,
    ) -> Result<Option<&Invocation<T, DID, V, C>>, <S as Store<T, DID, V, C>>::InvocationStoreError>
    {
        (*self).get(cid)
    }

    fn put(
        &mut self,
        cid: Cid,
        invocation: Invocation<T, DID, V, C>,
    ) -> Result<(), <S as Store<T, DID, V, C>>::InvocationStoreError> {
        (*self).put(cid, invocation)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStore<
    T = crate::ability::preset::Preset,
    DID: crate::did::Did = crate::did::preset::Verifier,
    V: varsig::Header<C> = varsig::header::Preset,
    C: Codec + TryFrom<u64> + Into<u64> = varsig::encoding::Preset,
> {
    store: BTreeMap<Cid, Invocation<T, DID, V, C>>,
}

impl<T, DID: Did, V: varsig::Header<Enc>, Enc: Codec + Into<u64> + TryFrom<u64>> Default
    for MemoryStore<T, DID, V, Enc>
{
    fn default() -> Self {
        Self {
            store: BTreeMap::new(),
        }
    }
}

impl<T, DID: Did, V: varsig::Header<Enc>, Enc: Codec + Into<u64> + TryFrom<u64>>
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
