use crate::ability::arguments;

pub trait Resolvable: Sized {
    type Promised: TryInto<Self> + From<Self> + Into<arguments::Named>;
}

// NOTE Promised into args should cover all of the values
