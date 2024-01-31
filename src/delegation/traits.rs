use crate::{ability::arguments::Arguments, did::Did, nonce::Nonce, task, task::Task};

pub trait Delegatable: Sized {
    type Builder: TryInto<Self> + From<Self> + Into<Arguments>;
}
