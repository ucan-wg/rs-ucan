use super::{Receipt, Responds};
use crate::task;
use libipld_core::ipld::Ipld;

pub trait Store {
    type Abilities: Responds;
    type Error;

    fn get(id: task::Id) -> Result<Receipt<Self::Abilities>, Self::Error>
    where
        <Self::Abilities as Responds>::Success: TryFrom<Ipld>;

    fn put_keyed(id: task::Id, receipt: Receipt<Self::Abilities>) -> Result<(), Self::Error>
    where
        <Self::Abilities as Responds>::Success: Into<Ipld>;
}
