use super::{any::AnyBuilder, mutate::MutateBuilder};
use crate::prove::traits::CheckSelf;

pub enum Mutable {
    Mutate(MutateBuilder),
    Any(AnyBuilder),
}

impl CheckSelf for Mutable {
    type SelfError = ();
    fn check_against_self(&self, other: &Self) -> Result<(), Self::SelfError> {
        match self {
            Mutable::Mutate(mutate) => match other {
                Mutable::Mutate(other_mutate) => mutate.check_against_self(other_mutate),
                Mutable::Any(any) => Ok(()),
            },
            _ => Err(()),
        }
    }
}
