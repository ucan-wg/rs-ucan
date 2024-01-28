use super::internal::IsChecker;

pub trait CheckSelf {
    type Error;

    fn check_against_self(&self, other: &Self) -> Result<(), Self::Error>;
}

pub trait CheckParents: CheckSelf {
    type Parents;
    type ParentError;

    fn check_against_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError>;
}

pub trait HasChecker: CheckSelf {
    type CheckAs: IsChecker;
}

pub trait Prove<T> {
    type ProveError;
    fn check<'a>(&'a self, other: &'a T) -> Result<(), Self::ProveError>;
}

// FIXME needed?
pub trait IntoParent<T: CheckParents> {
    fn as_parent(self) -> T::Parents;
}

// Nightly only... sadness
// trait Foo = HasChecker + Prove<Self::CheckAs>;
