use crate::{
    crypto::varsig,
    did::Did,
    receipt::{Receipt, Responds},
    task,
};
use libipld_core::{codec::Codec, ipld::Ipld};

/// A store for [`Receipt`]s indexed by their [`task::Id`]s.
pub trait Store<T: Responds, DID: Did, V: varsig::Header<C>, C: Codec + Into<u32> + TryFrom<u32>> {
    /// The error type representing all the ways a store operation can fail.
    type Error;

    /// Retrieve a [`Receipt`] by its [`task::Id`].
    ///
    /// If the store itself did not experience an error, but the value
    /// was not found, the result will be `Ok(None)`.
    fn get<'a>(&self, id: &task::Id) -> Result<Option<&Receipt<T, DID, V, C>>, Self::Error>
    where
        <T as Responds>::Success: TryFrom<Ipld>;

    /// Store a [`Receipt`] by its [`task::Id`].
    fn put(&mut self, id: task::Id, receipt: Receipt<T, DID, V, C>) -> Result<(), Self::Error>
    where
        <T as Responds>::Success: Into<Ipld>;
}
