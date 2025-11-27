use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::HashMap,
    convert::Infallible,
    future::Future,
    rc::Rc,
    sync::{Arc, Mutex},
};

use ipld_core::cid::Cid;
use thiserror::Error;
use varsig::verify::Verify;

use crate::did::Did;

use super::Delegation;

pub trait LocalDelegationStore<D: Did, T: Borrow<Delegation<D>>> {
    type LocalInsertError;
    type LocalGetError;

    fn local_get(&self, cid: &Cid) -> impl Future<Output = Result<Option<T>, Self::LocalGetError>>;
    fn local_insert_by_cid(
        &self,
        cid: Cid,
        delegation: T,
    ) -> impl Future<Output = Result<(), Self::LocalInsertError>>;

    fn local_insert(
        &self,
        delegation: T,
    ) -> impl Future<Output = Result<Cid, Self::LocalInsertError>> {
        async {
            let cid = delegation.borrow().to_cid();
            self.local_insert_by_cid(cid, delegation);
            Ok(cid)
        }
    }
}

pub trait DelegationStore<D: Did, T: Borrow<Delegation<D>> + Send>: Sync {
    type InsertError;
    type GetError;

    fn get(&self, cid: &Cid) -> impl Future<Output = Result<Option<T>, Self::GetError>> + Send;
    fn insert_by_cid(
        &self,
        cid: Cid,
        delegation: T,
    ) -> impl Future<Output = Result<(), Self::InsertError>> + Send;

    fn insert(&self, delegation: T) -> impl Future<Output = Result<Cid, Self::InsertError>> + Send {
        async {
            let cid = delegation.borrow().to_cid();
            self.insert_by_cid(cid, delegation);
            Ok(cid)
        }
    }
}

impl<D: Did> LocalDelegationStore<D, Rc<Delegation<D>>>
    for Rc<RefCell<HashMap<Cid, Rc<Delegation<D>>>>>
{
    type LocalInsertError = Infallible;
    type LocalGetError = Infallible;

    async fn local_insert_by_cid(
        &self,
        cid: Cid,
        delegation: Rc<Delegation<D>>,
    ) -> Result<(), Self::LocalInsertError> {
        self.borrow_mut().insert(cid, delegation);
        Ok(())
    }

    async fn local_get(&self, cid: &Cid) -> Result<Option<Rc<Delegation<D>>>, Self::LocalGetError> {
        Ok(RefCell::borrow(self).get(cid).cloned())
    }
}

impl<D: Did> LocalDelegationStore<D, Arc<Delegation<D>>>
    for Arc<Mutex<HashMap<Cid, Arc<Delegation<D>>>>>
{
    type LocalInsertError = StorePoisoned;
    type LocalGetError = StorePoisoned;

    async fn local_insert_by_cid(
        &self,
        cid: Cid,
        delegation: Arc<Delegation<D>>,
    ) -> Result<(), Self::LocalInsertError> {
        let mut locked = self.lock().map_err(|_| StorePoisoned)?;
        locked.insert(cid, delegation);
        Ok(())
    }

    async fn local_get(
        &self,
        cid: &Cid,
    ) -> Result<Option<Arc<Delegation<D>>>, Self::LocalGetError> {
        let locked = self.lock().map_err(|_| StorePoisoned)?;
        Ok(locked.get(cid).cloned())
    }
}

impl<D: Did + Send + Sync> DelegationStore<D, Arc<Delegation<D>>>
    for Arc<Mutex<HashMap<Cid, Arc<Delegation<D>>>>>
where
    <D as Did>::VarsigConfig: Send + Sync,
    <<D as Did>::VarsigConfig as Verify>::Signature: Send + Sync,
{
    type InsertError = StorePoisoned;
    type GetError = StorePoisoned;

    async fn insert_by_cid(
        &self,
        cid: Cid,
        delegation: Arc<Delegation<D>>,
    ) -> Result<(), Self::InsertError> {
        let mut locked = self.lock().map_err(|_| StorePoisoned)?;
        locked.insert(cid, delegation);
        Ok(())
    }

    async fn get(&self, cid: &Cid) -> Result<Option<Arc<Delegation<D>>>, Self::GetError> {
        let locked = self.lock().map_err(|_| StorePoisoned)?;
        Ok(locked.get(cid).cloned())
    }
}

/// Error for when the delegation store's [`Mutex`] is poisoned.
#[derive(Debug, Clone, Copy, Error)]
#[error("delegation store poisoned")]
pub struct StorePoisoned;
