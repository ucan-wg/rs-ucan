use crate::did::Did;
use serde::Serialize;

pub trait CheckSame {
    type Error;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error>;
}

// Genereic
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unequal;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum OptionalFieldErr {
    MissingField,
    UnequalValue,
}

impl CheckSame for Did {
    type Error = Unequal;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        if self.eq(proof) {
            Ok(())
        } else {
            Err(Unequal)
        }
    }
}

impl<T: PartialEq> CheckSame for Option<T> {
    type Error = OptionalFieldErr;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        match proof {
            None => Ok(()),
            Some(proof_) => match self {
                None => Err(OptionalFieldErr::MissingField),
                Some(self_) => {
                    if self_.eq(proof_) {
                        Ok(())
                    } else {
                        Err(OptionalFieldErr::UnequalValue)
                    }
                }
            },
        }
    }
}
