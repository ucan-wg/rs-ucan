use crate::prove::TryProve;
use void::Void;

pub struct DelegateAny;

impl<T> TryProve<DelegateAny> for T {
    type Error = Void;
    type Proven = T;

    fn try_prove<'a>(&'a self, _proof: &'a DelegateAny) -> Result<&'a Self::Proven, Void> {
        Ok(self)
    }
}
