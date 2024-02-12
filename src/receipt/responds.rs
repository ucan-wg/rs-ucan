use crate::{did::Did, nonce::Nonce, task, task::Task};
use libipld_core::ipld::Ipld;

/// Describe the relationship between an ability and the [`Receipt`]s.
///
/// This is used for constucting [`Receipt`]s, and indexing them for
/// reverse lookup.
///
/// [`Receipt`]: crate::receipt::Receipt
pub trait Responds {
    /// The successful return type for running `Self`.
    type Success;

    /// Convert an Ability (`Self`) into a [`Task`].
    ///
    /// This is used to index receipts by a minimal [`Id`].
    fn to_task(&self, subject: Did, nonce: Nonce) -> Task;

    /// Convert an Ability (`Self`) directly into a [`Task`]'s [`Id`].
    fn to_task_id(&self, subject: Did, nonce: Nonce) -> task::Id {
        task::Id {
            cid: self.to_task(subject, nonce).into(),
        }
    }
}
