use crate::{
    ability::traits::Command,
    prove::{
        parentless::Parentless,
        traits::{CheckSelf, Checkable},
    },
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

impl CheckSelf for AnyBuilder {
    type Error = ();
    fn check_against_self(&self, _proof: &Self) -> Result<(), Self::Error> {
        Ok(())
    }
}
