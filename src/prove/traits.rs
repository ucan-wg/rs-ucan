use super::internal::Checker;

pub trait CheckSelf {
    type Error;

    fn check_against_self(&self, proof: &Self) -> Result<(), Self::Error>;
}

pub trait CheckParents: CheckSelf {
    type Parents;
    type ParentError;

    fn check_against_parents(&self, proof: &Self::Parents) -> Result<(), Self::ParentError>;
}

pub trait Checkable: CheckSelf {
    type CheckAs: Checker;
}

// FIXME is it worth locking consumers out with that Checker bound?
pub trait Prove<T: Checker> {
    type ProveError;
    fn check<'a>(&'a self, proof: &'a T) -> Result<(), Self::ProveError>;
}

// Nightly only... sadness
// trait Foo = Checkable + Prove<Self::CheckAs>;
