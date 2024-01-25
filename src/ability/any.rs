use crate::prove::TryProve;
use std::convert::Infallible;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DelegateAny;

impl<'a, T> TryProve<'a, DelegateAny> for T {
    type Error = Infallible;
    type Proven = T;

    fn try_prove(&'a self, _proof: &'a DelegateAny) -> Result<&'a Self::Proven, Infallible> {
        Ok(self)
    }
}
