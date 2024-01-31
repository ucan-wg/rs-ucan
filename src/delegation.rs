// FIXME rename delegate?

pub mod condition;
pub mod delegatable;
pub mod delegate;
pub mod payload;

use crate::signature;
pub use delegate::Delegate;
pub use payload::Payload;

/// A [`Delegation`] is a signed delegation [`Payload`]
///
/// A [`Payload`] on its own is not a valid [`Delegation`], as it must be signed by the issuer.
///
/// # Examples
/// FIXME
pub type Delegation<B, C> = signature::Envelope<Payload<B, C>>;
