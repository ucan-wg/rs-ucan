use super::{internal::Checker, same::CheckSame};

pub trait Checkable: CheckSame {
    type CheckAs: Checker;
}
