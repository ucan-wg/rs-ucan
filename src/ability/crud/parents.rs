use super::{any::AnyBuilder, mutate::MutateBuilder};
use crate::prove::traits::CheckSelf;

pub enum Mutable {
    Mutate(MutateBuilder),
    Any(AnyBuilder),
}

impl CheckSelf for Mutable {
    type Error = ();
    fn check_against_self(&self, other: &Self) -> Result<(), Self::Error> {
        match self {
            Mutable::Mutate(mutate) => match other {
                Mutable::Mutate(other_mutate) => mutate.check_against_self(other_mutate),
                Mutable::Any(any) => Ok(()),
            },
            _ => Err(()),
        }
    }
}
