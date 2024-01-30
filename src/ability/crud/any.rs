use crate::{
    ability::traits::Command,
    proof::{
        parentless::NoParents,
        same::{CheckSame, OptionalFieldErr},
    },
};
use serde::{Deserialize, Serialize};
use url::Url;

// NOTE no resolved or awaiting variants, because this cannot be executed, and all fields are optional already!

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Builder {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Url>,
}

impl Command for Builder {
    const COMMAND: &'static str = "crud/*";
}

impl NoParents for Builder {}

impl CheckSame for Builder {
    type Error = OptionalFieldErr;
    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        self.uri.check_same(&proof.uri)
    }
}
