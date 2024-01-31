use super::same::CheckSame;

pub trait CheckParents: CheckSame {
    type Parents;
    type ParentError;

    fn check_parents(&self, proof: &Self::Parents) -> Result<(), Self::ParentError>;
}