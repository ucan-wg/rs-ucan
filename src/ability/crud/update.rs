use crate::{
    ability::traits::Command,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use url::Url;

use super::parents::Mutable;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Update {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Url>,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub args: BTreeMap<String, Ipld>,
}

impl From<Update> for Ipld {
    fn from(udpdate: Update) -> Self {
        udpdate.into()
    }
}

impl TryFrom<Ipld> for Update {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

impl Command for Update {
    const COMMAND: &'static str = "crud/update";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateBuilder {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Url>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<BTreeMap<String, Ipld>>, // FIXME use a type param?
}

impl From<UpdateBuilder> for Ipld {
    fn from(udpdate: UpdateBuilder) -> Self {
        udpdate.into()
    }
}

impl TryFrom<Ipld> for UpdateBuilder {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

impl Checkable for UpdateBuilder {
    type CheckAs = Parentful<UpdateBuilder>;
}

impl CheckSame for UpdateBuilder {
    type Error = (); // FIXME

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        self.uri.check_same(&proof.uri).map_err(|_| ())?;
        self.args.check_same(&proof.args).map_err(|_| ())
    }
}

impl CheckParents for UpdateBuilder {
    type Parents = Mutable;
    type ParentError = (); // FIXME

    fn check_parents(&self, proof: &Self::Parents) -> Result<(), Self::ParentError> {
        match proof {
            Mutable::Any(any) => self.uri.check_same(&any.uri).map_err(|_| ()),
            Mutable::Mutate(mutate) => self.uri.check_same(&mutate.uri).map_err(|_| ()),
        }
    }
}
