use super::{any::AnyBuilder, mutate::MutateBuilder};
use crate::proof::same::CheckSame;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Mutable {
    Mutate(MutateBuilder),
    Any(AnyBuilder),
}

impl CheckSame for Mutable {
    type Error = ();
    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        match self {
            Mutable::Mutate(mutate) => match proof {
                Mutable::Mutate(other_mutate) => mutate.check_same(other_mutate),
                Mutable::Any(any) => Ok(()),
            },
            _ => Err(()),
        }
    }
}
