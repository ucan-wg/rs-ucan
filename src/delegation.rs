mod condition;
mod delegatable;
mod payload;

pub mod store;

pub use condition::*;
pub use delegatable::Delegatable;
pub use payload::Payload;

use condition::traits::Condition;
use store::IndexedStore;

use crate::signature;

/// A [`Delegation`] is a signed delegation [`Payload`]
///
/// A [`Payload`] on its own is not a valid [`Delegation`], as it must be signed by the issuer.
///
/// # Examples
/// FIXME
pub type Delegation<T, C> = signature::Envelope<Payload<T, C>>;

// FIXME
impl<T: Delegatable, C: Condition> Delegation<T, C> {
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
