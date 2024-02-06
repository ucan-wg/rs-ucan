use super::{Receipt, Responds};
use crate::{metadata, task};
use libipld_core::ipld::Ipld;

pub trait Store<T: Responds, E: metadata::MultiKeyed> {
    type Error;

    fn get(id: task::Id) -> Result<Receipt<T, E>, Self::Error>
    where
        <T as Responds>::Success: TryFrom<Ipld>;

    fn put_keyed(id: task::Id, receipt: Receipt<T, E>) -> Result<(), Self::Error>
    where
        <T as Responds>::Success: Into<Ipld>;
}
