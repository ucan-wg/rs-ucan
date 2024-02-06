use crate::{ability::arguments, did::Did, nonce::Nonce, task, task::Task};

pub trait Delegatable: Sized {
    type Builder: TryInto<Self> + From<Self> + Into<Named>;
}
