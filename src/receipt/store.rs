use super::{Receipt, Responds};
use crate::task;
use libipld_core::ipld::Ipld;

pub trait Store<T: Responds> {
    type Error;

    fn get(id: task::Id) -> Result<Receipt<T>, Self::Error>
    where
        <T as Responds>::Success: TryFrom<Ipld>;

    fn put_keyed(id: task::Id, receipt: Receipt<T>) -> Result<(), Self::Error>
    where
        <T as Responds>::Success: Into<Ipld>;
}
