use crate::ability::arguments;

pub trait Delegatable: Sized {
    /// A delegation with some arguments filled
    /// FIXME add more
    type Builder: TryInto<Self> + From<Self> + Into<arguments::Named>;
}
