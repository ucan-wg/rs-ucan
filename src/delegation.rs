mod delegable;
mod payload;

pub mod condition;
pub mod error;
pub mod store;

pub use delegable::Delegable;
pub use payload::Payload;

use crate::proof::{checkable::Checkable, parents::CheckParents, same::CheckSame};
use condition::Condition;

use crate::signature;

/// A [`Delegation`] is a signed delegation [`Payload`]
///
/// A [`Payload`] on its own is not a valid [`Delegation`], as it must be signed by the issuer.
///
/// # Examples
/// FIXME
pub type Delegation<T, C> = signature::Envelope<Payload<T, C>>;

impl<T: CheckSame, C: Condition> CheckSame for Delegation<T, C> {
    type Error = <T as CheckSame>::Error;

    fn check_same(&self, proof: &Delegation<T, C>) -> Result<(), Self::Error> {
        self.payload.check_same(&proof.payload)
    }
}

impl<T: CheckParents, C: Condition> CheckParents for Delegation<T, C> {
    type Parents = Delegation<T::Parents, C>;
    type ParentError = <T as CheckParents>::ParentError;

    fn check_parent(&self, proof: &Self::Parents) -> Result<(), Self::ParentError> {
        self.payload.check_parent(&proof.payload)
    }
}

// // FIXME relax the checkable constraint for this? Or make this an instance of checker?
// impl<T: Checkable<Hierarchy = Parentful<U>>, C> Checkable for Delegation<T, C> {
//     type Hierarchy = Parentful<Delegation<T::Hierarchy>, C>;
// }

// FIXME
impl<T: Delegable, C: Condition> Delegation<T, C> {
    // FIXME include cache
    //pub fn check<S: IndexedStore<T, C, E>>(&self, store: &S) -> Result<(), ()> {
    //    if let Ok(is_valid) = store.previously_checked(self) {
    //        if is_valid {
    //            return Ok(());
    //        }
    //    }

    //    if let Ok(chains) = store.chains_for(self) {
    //        for chain in chains {
    //            todo!()
    //            // if self.check_self(self).is_ok() {
    //            //     return Ok(());
    //            // }
    //        }
    //    }

    //    Err(())
    //}
}
