use crate::{
    ability::traits::Command,
    proof::{checkable::Checkable, parentless::Parentless, same::CheckSame},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Any {
    pub from: Option<Url>,
}

impl Command for Any {
    const COMMAND: &'static str = "msg";
}

impl From<Any> for Ipld {
    fn from(any: Any) -> Self {
        any.into()
    }
}

impl TryFrom<Ipld> for Any {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Checkable for Any {
    type Heirarchy = Parentless<Any>;
}

impl CheckSame for Any {
    type Error = ();
    fn check_same(&self, _proof: &Self) -> Result<(), Self::Error> {
        Ok(())
    }
}
