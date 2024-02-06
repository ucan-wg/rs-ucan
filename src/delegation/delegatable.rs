use crate::ability::arguments;

pub trait Delegatable: Sized {
    type Builder: TryInto<Self> + From<Self> + Into<arguments::Named>;
}
