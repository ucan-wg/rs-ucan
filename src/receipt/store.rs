use super::{Receipt, Responds};
use crate::task;
use libipld_core::ipld::Ipld;

/// A store for [`Receipt`]s indexed by their [`task::Id`]s.
pub trait Store<T: Responds> {
    /// The error type representing all the ways a store operation can fail.
    type Error;

    /// Retrieve a [`Receipt`] by its [`task::Id`].
    fn get(id: task::Id) -> Result<Receipt<T>, Self::Error>
    where
        <T as Responds>::Success: TryFrom<Ipld>;

    /// Store a [`Receipt`] by its [`task::Id`].
    fn put_keyed(id: task::Id, receipt: Receipt<T>) -> Result<(), Self::Error>
    where
        <T as Responds>::Success: Into<Ipld>;
}
