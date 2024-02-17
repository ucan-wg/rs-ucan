//! A [`Receipt`] is the (optional) response from an [`Invocation`][`crate::invocation::Invocation`].
//!
//! - [`Receipt`]s are the result of an [`Invocation`][`crate::invocation::Invocation`].
//! - [`Payload`] contains the pimary semantic information for a [`Receipt`].
//! - [`Store`] is the storage interface for [`Receipt`]s.
//! - [`Responds`] associates the response success type to an [Ability][crate::ability].

mod payload;
mod responds;

pub mod store;

pub use payload::Payload;
pub use responds::Responds;
pub use store::Store;

use crate::{ability, did, signature};

/// The complete, signed receipt of an [`Invocation`][`crate::invocation::Invocation`].
#[derive(Clone, Debug, PartialEq)]
pub struct Receipt<T: Responds, DID: did::Did>(pub signature::Envelope<Payload<T, DID>, DID>);

/// An alias for the [`Receipt`] type with the library preset
/// [`Did`](crate::did)s and [Abilities](crate::ability).
pub type Preset = Receipt<ability::preset::Ready, did::preset::Verifier>;

impl<T: Responds, DID: did::Did> Receipt<T, DID> {
    /// Returns the [`Payload`] of the [`Receipt`].
    pub fn payload(&self) -> &Payload<T, DID> {
        &self.0.payload
    }

    /// Returns the [`signature::Envelope`] of the [`Receipt`].
    pub fn signature(&self) -> &signature::Witness<DID::Signature> {
        &self.0.signature
    }
}
