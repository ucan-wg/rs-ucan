use crate::ability::arguments::Arguments;

pub trait Delegatable: Sized {
    type Builder: TryInto<Self> + From<Self> + Into<Arguments>;
}
