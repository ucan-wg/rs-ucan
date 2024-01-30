use super::{internal::Checker, prove::Prove, same::CheckSame};

pub trait Checkable: CheckSame {
    type Hierarchy: Checker + Prove<Self::Hierarchy>;
}
