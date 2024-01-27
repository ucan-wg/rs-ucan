use crate::prove::TryProve;
use std::convert::Infallible;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DelegateAny;

impl<'a, T> TryProve<&'a DelegateAny> for &'a T {
    type Error = Infallible;
    type Proven = &'a T;

    fn try_prove(self, _proof: &'a DelegateAny) -> Result<Self::Proven, Infallible> {
        Ok(self)
    }
}
