use super::{internal::Checker, prove::Prove, same::CheckSame};

pub trait Checkable: CheckSame {
    type Heirarchy: Checker + Prove<Self::Heirarchy>;
}
