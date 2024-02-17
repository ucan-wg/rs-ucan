//! The (optional) response from an [`Invocation`][`crate::invocation::Invocation`].

mod payload;
mod responds;

pub mod store;

pub use payload::Payload;
pub use responds::Responds;

use crate::{ability, did, signature};

/// The complete, signed receipt of an [`Invocation`][`crate::invocation::Invocation`].
pub type Receipt<T, DID> = signature::Envelope<Payload<T, DID>, DID>;

pub type Preset = Receipt<ability::preset::Ready, did::Preset>;
