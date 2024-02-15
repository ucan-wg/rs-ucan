use crate::{ability::arguments, delegation::Delegable};
use libipld_core::ipld::Ipld;

pub trait Resolvable: Delegable {
    type Promised: Into<arguments::Named<Ipld>>;

    // FIXME indeed needed to get teh right err type
    fn try_resolve(promised: Self::Promised) -> Result<Self, Self::Promised>;
}
