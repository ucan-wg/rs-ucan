use crate::prove::TryProve;
use void::Void;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DelegateAny;

impl<'a, T> TryProve<'a, DelegateAny> for T {
    type Error = Void;
    type Proven = T;

    fn try_prove(&'a self, _proof: &'a DelegateAny) -> Result<&'a Self::Proven, Void> {
        Ok(self)
    }
}
