use super::{internal::Checker, prove::Prove, same::CheckSame};

pub trait Checkable: CheckSame {
    type Hierarchy: Checker + CheckSame + Prove<Self::Hierarchy>;
}
