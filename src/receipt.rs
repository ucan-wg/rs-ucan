//! The (optional) response from an [`Invocation`][`crate::invocation::Invocation`].

mod payload;
mod responds;
mod store;

pub use payload::Payload;
pub use responds::Responds;
pub use store::Store;

use crate::signature;

/// The complete, signed receipt of an [`Invocation`][`crate::invocation::Invocation`].
pub type Receipt<T> = signature::Envelope<Payload<T>>;
