use crate::{crypto::varsig, did::Did, invocation::Invocation};
use libipld_core::{cid::Cid, codec::Codec};
use std::sync::Arc;

pub trait Store<T, DID: Did, V: varsig::Header<C>, C: Codec + Into<u64> + TryFrom<u64>> {
    type InvocationStoreError;

    fn get(
        &self,
        cid: Cid,
    ) -> Result<Option<Arc<Invocation<T, DID, V, C>>>, Self::InvocationStoreError>;

    fn put(
        &self,
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
    > Store<T, DID, V, C> for &S
{
    type InvocationStoreError = <S as Store<T, DID, V, C>>::InvocationStoreError;

    fn get(
        &self,
        cid: Cid,
    ) -> Result<
        Option<Arc<Invocation<T, DID, V, C>>>,
        <S as Store<T, DID, V, C>>::InvocationStoreError,
    > {
        (**self).get(cid)
    }

    fn put(
        &self,
        cid: Cid,
        invocation: Invocation<T, DID, V, C>,
    ) -> Result<(), <S as Store<T, DID, V, C>>::InvocationStoreError> {
        (**self).put(cid, invocation)
    }
}
