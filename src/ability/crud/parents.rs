use super::{any, mutate::MutateBuilder};
use crate::proof::same::CheckSame;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Mutable {
    Mutate(MutateBuilder),
    Any(any::Builder),
}

impl CheckSame for Mutable {
    type Error = ();
    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        match self {
            Mutable::Mutate(mutate) => match proof {
                Mutable::Mutate(other_mutate) => mutate.check_same(other_mutate),
                Mutable::Any(_any) => Ok(()),
            },
            _ => Err(()),
        }
    }
}
