use crate::proof::checkable::Checkable;

pub trait Delegable: Sized {
    /// A delegation with some arguments filled
    /// FIXME add more text
    type Builder: TryInto<Self> + From<Self> + Checkable;
}
