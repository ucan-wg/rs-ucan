use crate::ability::arguments;

pub trait Resolvable: Sized {
    type Promised: TryInto<Self> + From<Self> + Into<arguments::Named>;
}
