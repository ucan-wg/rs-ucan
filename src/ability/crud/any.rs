use crate::{
    ability::traits::Command,
    proof::{checkable::Checkable, parentless::Parentless, same::CheckSame},
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnyBuilder {
    pub uri: Option<Url>,
}

impl Command for AnyBuilder {
    const COMMAND: &'static str = "crud/*";
}

impl Checkable for AnyBuilder {
    type CheckAs = Parentless<AnyBuilder>;
}

impl CheckSame for AnyBuilder {
    type Error = ();
    fn check_same(&self, _proof: &Self) -> Result<(), Self::Error> {
        Ok(())
    }
}
