use crate::{crypto::Nonce, task, task::Task};
use std::fmt;

/// Describe the relationship between an ability and the [`Receipt`]s.
///
/// This is used for constucting [`Receipt`]s, and indexing them for
/// reverse lookup.
///
/// [`Receipt`]: crate::receipt::Receipt
pub trait Responds {
    /// The successful return type for running `Self`.
    type Success: Clone + fmt::Debug + PartialEq;

    /// Convert an Ability (`Self`) into a [`Task`].
    ///
    /// This is used to index receipts by a minimal [`Id`].
    fn to_task(&self, subject: did_url::DID, nonce: Nonce) -> Task;

    /// Convert an Ability (`Self`) directly into a [`Task`]'s [`Id`].
    fn to_task_id(&self, subject: did_url::DID, nonce: Nonce) -> task::Id {
        task::Id {
            cid: self.to_task(subject, nonce).into(),
        }
    }
}
