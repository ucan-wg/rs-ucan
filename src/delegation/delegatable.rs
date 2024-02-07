use crate::ability::arguments;

// FIXME require checkable?
pub trait Delegatable: Sized {
    /// A delegation with some arguments filled
    /// FIXME add more
    /// FIXME require CheckSame?
    type Builder: TryInto<Self> + From<Self> + Into<arguments::Named>;
}
