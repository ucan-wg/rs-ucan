use crate::prove::{
    parentless::Parentless,
    traits::{CheckSelf, HasChecker},
};
use url::Url;

pub struct AnyBuilder {
    pub uri: Option<Url>,
}

impl HasChecker for AnyBuilder {
    type CheckAs = Parentless<AnyBuilder>;
}

impl CheckSelf for AnyBuilder {
    type Error = ();
    fn check_against_self(&self, _other: &Self) -> Result<(), Self::Error> {
        Ok(())
    }
}
