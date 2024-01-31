// FIXME rename delegate?

mod condition;
mod delegatable;
mod payload;

pub use condition::traits::Condition;
pub use delegatable::Delegatable;
pub use payload::Payload;

use crate::signature;

/// A [`Delegation`] is a signed delegation [`Payload`]
///
/// A [`Payload`] on its own is not a valid [`Delegation`], as it must be signed by the issuer.
///
/// # Examples
/// FIXME
pub type Delegation<B, C> = signature::Envelope<Payload<B, C>>;
