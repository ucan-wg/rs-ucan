use crate::{crypto::varsig, did::Did, invocation::Invocation};
use super::Store;
use libipld_core::{cid::Cid, codec::Codec};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{collections::BTreeMap, convert::Infallible};

#[derive(Debug, Clone)]
pub struct MemoryStore<
    T = crate::ability::preset::Preset,
    DID: crate::did::Did = crate::did::preset::Verifier,
    V: varsig::Header<C> = varsig::header::Preset,
    C: Codec + TryFrom<u64> + Into<u64> = varsig::encoding::Preset,
> {
    inner: Arc<RwLock<MemoryStoreInner<T, DID, V, C>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStoreInner<
    T = crate::ability::preset::Preset,
    DID: crate::did::Did = crate::did::preset::Verifier,
    V: varsig::Header<C> = varsig::header::Preset,
    C: Codec + TryFrom<u64> + Into<u64> = varsig::encoding::Preset,
> {
    store: BTreeMap<Cid, Arc<Invocation<T, DID, V, C>>>,
}

impl<T, DID: Did, V: varsig::Header<Enc>, Enc: Codec + Into<u64> + TryFrom<u64>>
    MemoryStore<T, DID, V, Enc>
{
    fn read(&self) -> RwLockReadGuard<'_, MemoryStoreInner<T, DID, V, Enc>> {
        match self.inner.read() {
            Ok(guard) => guard,
            Err(poison) => {
                // There's no logic errors through lock poisoning in our case
                poison.into_inner()
            }
        }
    }

    fn write(&self) -> RwLockWriteGuard<'_, MemoryStoreInner<T, DID, V, Enc>> {
        match self.inner.write() {
            Ok(guard) => guard,
            Err(poison) => {
                // There's no logic errors through lock poisoning in our case
                poison.into_inner()
            }
        }
    }
}

impl<T, DID: Did, V: varsig::Header<Enc>, Enc: Codec + Into<u64> + TryFrom<u64>> Default
    for MemoryStore<T, DID, V, Enc>
{
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MemoryStoreInner {
                store: BTreeMap::new(),
            })),
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
    ) -> Result<Option<Arc<Invocation<T, DID, V, Enc>>>, Self::InvocationStoreError> {
        Ok(self.read().store.get(&cid).cloned())
    }

    fn put(
        &self,
        cid: Cid,
        invocation: Invocation<T, DID, V, Enc>,
    ) -> Result<(), Self::InvocationStoreError> {
        self.write().store.insert(cid, Arc::new(invocation));
        Ok(())
    }
}
