use crate::ability::arguments::Arguments;

pub trait Resolvable: Sized {
    type Promised: TryInto<Self> + From<Self> + Into<Arguments>;
}
