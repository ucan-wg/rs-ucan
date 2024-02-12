use crate::{ability::arguments, proof::checkable::Checkable};
use libipld_core::ipld::Ipld;

// FIXME require checkable?
pub trait Delegatable: Sized {
    /// A delegation with some arguments filled
    /// FIXME add more
    type Builder: TryInto<Self> + From<Self> + Checkable;
}
