//! Delegation stores.

use alloc::{rc::Rc, vec::Vec};
use core::{borrow::Borrow, cell::RefCell, convert::Infallible, error::Error};

use alloc::collections::BTreeMap;
use future_form::{FutureForm, Local};
use ipld_core::cid::Cid;
use thiserror::Error;

use crate::did::Did;

use super::Delegation;

#[cfg(feature = "std")]
use {
    alloc::sync::Arc,
    future_form::{future_form, Sendable},
    std::{collections::HashMap, hash::BuildHasher, sync::Mutex},
    varsig::verify::Verify,
};

/// Delegation store.
pub trait DelegationStore<K: FutureForm, D: Did, T: Borrow<Delegation<D>>> {
    /// Error type for insertion operations.
    type InsertError: Error;

    /// Error type for retrieval operations.
    type GetError: Error;

    /// Retrieves delegations by their CIDs.
    fn get_all<'a>(&'a self, cid: &'a [Cid]) -> K::Future<'a, Result<Vec<T>, Self::GetError>>;

    /// Inserts a delegation by its CID.
    fn insert_by_cid(
        &self,
        cid: Cid,
        delegation: T,
    ) -> K::Future<'_, Result<(), Self::InsertError>>;
}

/// Inserts a delegation and returns its CID.
///
/// # Errors
///
/// If insertion fails, an error defined by the `impl DelegationStore` is returned
/// (the `S::InsertError` associated type).
pub async fn insert<
    K: FutureForm,
    D: Did,
    T: Borrow<Delegation<D>>,
    S: DelegationStore<K, D, T>,
>(
    store: &S,
    delegation: T,
) -> Result<Cid, S::InsertError> {
    let cid = delegation.borrow().to_cid();
    store.insert_by_cid(cid, delegation).await?;
    Ok(cid)
}

// ---------------------------------------------------------------------------
// no_std: Rc<RefCell<BTreeMap>> store
// ---------------------------------------------------------------------------

impl<D: Did> DelegationStore<Local, D, Rc<Delegation<D>>>
    for Rc<RefCell<BTreeMap<Cid, Rc<Delegation<D>>>>>
{
    type InsertError = Infallible;
    type GetError = Missing;

    fn insert_by_cid(
        &self,
        cid: Cid,
        delegation: Rc<Delegation<D>>,
    ) -> <Local as FutureForm>::Future<'_, Result<(), Self::InsertError>> {
        Local::from_future(async move {
            self.borrow_mut().insert(cid, delegation);
            Ok(())
        })
    }

    fn get_all<'a>(
        &'a self,
        cid: &'a [Cid],
    ) -> <Local as FutureForm>::Future<'a, Result<Vec<Rc<Delegation<D>>>, Self::GetError>> {
        Local::from_future(async move {
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
        })
    }
}

// ---------------------------------------------------------------------------
// std: Rc<RefCell<HashMap>> store
// ---------------------------------------------------------------------------

#[cfg(feature = "std")]
impl<D: Did, H: BuildHasher> DelegationStore<Local, D, Rc<Delegation<D>>>
    for Rc<RefCell<HashMap<Cid, Rc<Delegation<D>>, H>>>
{
    type InsertError = Infallible;
    type GetError = Missing;

    fn insert_by_cid(
        &self,
        cid: Cid,
        delegation: Rc<Delegation<D>>,
    ) -> <Local as FutureForm>::Future<'_, Result<(), Self::InsertError>> {
        Local::from_future(async move {
            self.borrow_mut().insert(cid, delegation);
            Ok(())
        })
    }

    fn get_all<'a>(
        &'a self,
        cid: &'a [Cid],
    ) -> <Local as FutureForm>::Future<'a, Result<Vec<Rc<Delegation<D>>>, Self::GetError>> {
        Local::from_future(async move {
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
        })
    }
}

// ---------------------------------------------------------------------------
// std: Arc<Mutex<HashMap>> store (Send + !Send variants)
// ---------------------------------------------------------------------------

#[cfg(feature = "std")]
#[future_form(
    Local,
    Sendable where
        D: Send + Sync,
        H: Send,
        <D as Did>::VarsigConfig: Send + Sync,
        <<D as Did>::VarsigConfig as Verify>::Signature: Send + Sync
)]
impl<K: FutureForm, D: Did, H: BuildHasher> DelegationStore<K, D, Arc<Delegation<D>>>
    for Arc<Mutex<HashMap<Cid, Arc<Delegation<D>>, H>>>
{
    type InsertError = StorePoisoned;
    type GetError = LockedStoreGetError;

    fn insert_by_cid(
        &self,
        cid: Cid,
        delegation: Arc<Delegation<D>>,
    ) -> K::Future<'_, Result<(), Self::InsertError>> {
        K::from_future(async move {
            let mut locked = self.lock().map_err(|_| StorePoisoned)?;
            locked.insert(cid, delegation);
            Ok(())
        })
    }

    fn get_all<'a>(
        &'a self,
        cid: &'a [Cid],
    ) -> K::Future<'a, Result<Vec<Arc<Delegation<D>>>, Self::GetError>> {
        K::from_future(async move {
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
        })
    }
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Error for when the delegation store's [`Mutex`] is poisoned.
#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, Error)]
#[error("delegation store poisoned")]
pub struct StorePoisoned;

/// Error for when a delegation is missing from the store.
#[derive(Debug, Clone, Copy, Error)]
#[error("delegation with cid {0} is missing")]
pub struct Missing(pub Cid);

/// Error for when the delegation store's [`Mutex`] is poisoned or a delegation is missing.
#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, Error)]
pub enum LockedStoreGetError {
    /// Delegation is missing
    #[error(transparent)]
    Missing(#[from] Missing),

    /// Mutex was poisoned
    #[error(transparent)]
    StorePoisoned(#[from] StorePoisoned),
}
