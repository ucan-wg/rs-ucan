mod condition;
mod delegatable;
mod payload;

pub use condition::*;

pub use delegatable::Delegatable;
pub use payload::Payload;

use crate::signature;

/// A [`Delegation`] is a signed delegation [`Payload`]
///
/// A [`Payload`] on its own is not a valid [`Delegation`], as it must be signed by the issuer.
///
/// # Examples
/// FIXME
pub type Delegation<T, C, E> = signature::Envelope<Payload<T, C, E>>;

// FIXME add a store with delegation indexing
