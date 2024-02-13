//! The (optional) response from an [`Invocation`][`crate::invocation::Invocation`].

mod payload;
mod responds;

pub mod store;

pub use payload::Payload;
pub use responds::Responds;

use crate::{ability, did, did::Did, signature};

/// The complete, signed receipt of an [`Invocation`][`crate::invocation::Invocation`].
pub type Receipt<T, DID: Did> = signature::Envelope<Payload<T, DID>, DID::Signature>;

pub type Preset = Receipt<ability::preset::Ready, did::Preset>;
