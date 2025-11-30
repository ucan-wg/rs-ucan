//! Delegation stores.

use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::HashMap,
    convert::Infallible,
    error::Error,
    rc::Rc,
    sync::{Arc, Mutex},
};

use futures::{
    future::{BoxFuture, LocalBoxFuture},
    FutureExt,
};
use ipld_core::cid::Cid;
use thiserror::Error;
use varsig::verify::Verify;

use crate::{
    did::Did,
    future::{FutureKind, Local, Sendable},
};

use super::Delegation;

/// Delegation store.
pub trait DelegationStore<K: FutureKind, D: Did, T: Borrow<Delegation<D>>> {
    /// Error type for insertion operations.
    type InsertError: Error;

    /// Error type for retrieval operations.
    type GetError: Error;

    /// Retrieves a delegation by its CID.
    fn get_all<'a>(&'a self, cid: &'a [Cid]) -> K::Future<'a, Result<Vec<T>, Self::GetError>>;

    /// Inserts a delegation by its CID.
    fn insert_by_cid<'a>(
        &'a self,
        cid: Cid,
        delegation: T,
    ) -> K::Future<'a, Result<(), Self::InsertError>>;

    /// Inserts a delegation and returns its CID.
    async fn insert(&self, delegation: T) -> Result<Cid, Self::InsertError> {
        let cid = delegation.borrow().to_cid();
        self.insert_by_cid(cid, delegation).await?;
        Ok(cid)
    }
}

impl<D: Did> DelegationStore<Local, D, Rc<Delegation<D>>>
    for Rc<RefCell<HashMap<Cid, Rc<Delegation<D>>>>>
{
    type InsertError = Infallible;
    type GetError = Missing;

    fn insert_by_cid<'a>(
        &'a self,
        cid: Cid,
        delegation: Rc<Delegation<D>>,
    ) -> LocalBoxFuture<'a, Result<(), Self::InsertError>> {
        async move {
            self.borrow_mut().insert(cid, delegation);
            Ok(())
        }
        .boxed_local()
    }

    fn get_all<'a>(
        &'a self,
        cid: &'a [Cid],
    ) -> LocalBoxFuture<'a, Result<Vec<Rc<Delegation<D>>>, Self::GetError>> {
        async move {
            let store = RefCell::borrow(self);
            let mut dlgs = Vec::new();
            for c in cid {
                if let Some(dlg) = store.get(c) {
                    dlgs.push(dlg.clone());
                } else {
                    Err(Missing(*c))?;
                }
            }
            Ok(dlgs)
        }
        .boxed_local()
    }
}

impl<D: Did> DelegationStore<Local, D, Arc<Delegation<D>>>
    for Arc<Mutex<HashMap<Cid, Arc<Delegation<D>>>>>
{
    type InsertError = StorePoisoned;
    type GetError = LockedStoreGetError;

    fn insert_by_cid<'a>(
        &'a self,
        cid: Cid,
        delegation: Arc<Delegation<D>>,
    ) -> LocalBoxFuture<'a, Result<(), Self::InsertError>> {
        async move {
            let mut locked = self.lock().map_err(|_| StorePoisoned)?;
            locked.insert(cid, delegation);
            Ok(())
        }
        .boxed_local()
    }

    fn get_all<'a>(
        &'a self,
        cid: &'a [Cid],
    ) -> LocalBoxFuture<'a, Result<Vec<Arc<Delegation<D>>>, Self::GetError>> {
        async move {
            let locked = self.lock().map_err(|_| StorePoisoned)?;
            let mut dlgs = Vec::new();
            for c in cid {
                if let Some(dlg) = locked.get(c) {
                    dlgs.push(dlg.clone());
                } else {
                    return Err(Missing(*c))?;
                }
            }
            Ok(dlgs)
        }
        .boxed_local()
    }
}

impl<D: Did + Send + Sync> DelegationStore<Sendable, D, Arc<Delegation<D>>>
    for Arc<Mutex<HashMap<Cid, Arc<Delegation<D>>>>>
where
    <D as Did>::VarsigConfig: Send + Sync,
    <<D as Did>::VarsigConfig as Verify>::Signature: Send + Sync,
{
    type InsertError = StorePoisoned;
    type GetError = LockedStoreGetError;

    fn insert_by_cid<'a>(
        &'a self,
        cid: Cid,
        delegation: Arc<Delegation<D>>,
    ) -> BoxFuture<'a, Result<(), Self::InsertError>> {
        async move {
            let mut locked = self.lock().map_err(|_| StorePoisoned)?;
            locked.insert(cid, delegation);
            Ok(())
        }
        .boxed()
    }

    fn get_all<'a>(
        &'a self,
        cid: &'a [Cid],
    ) -> BoxFuture<'a, Result<Vec<Arc<Delegation<D>>>, Self::GetError>> {
        async move {
            let locked = self.lock().map_err(|_| StorePoisoned)?;
            let mut dlgs = Vec::new();
            for c in cid {
                if let Some(dlg) = locked.get(c) {
                    dlgs.push(dlg.clone());
                } else {
                    return Err(Missing(*c))?;
                }
            }
            Ok(dlgs)
        }
        .boxed()
    }
}

#[derive(Debug, Clone, Copy, Error)]
#[error("delegation store poisoned")]
pub struct StorePoisoned;

#[derive(Debug, Clone, Copy, Error)]
#[error("delegation with cid {0} is missing")]
pub struct Missing(pub Cid);

/// Error for when the delegation store's [`Mutex`] is poisoned.
#[derive(Debug, Clone, Copy, Error)]
pub enum LockedStoreGetError {
    #[error(transparent)]
    Missing(#[from] Missing),

    #[error(transparent)]
    StorePoisoned(#[from] StorePoisoned),
}
