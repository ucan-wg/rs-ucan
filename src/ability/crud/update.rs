use crate::{
    ability::traits::Command,
    prove::{
        parentful::Parentful,
        traits::{CheckParents, CheckSelf, Checkable},
    },
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

impl CheckSelf for UpdateBuilder {
    type Error = ();
    fn check_against_self(&self, _proof: &Self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl CheckParents for UpdateBuilder {
    type Parents = Mutable;
    type ParentError = ();

    fn check_against_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        if let Some(self_uri) = &self.uri {
            match other {
                Mutable::Any(any) => {
                    if let Some(proof_uri) = &any.uri {
                        if self_uri != proof_uri {
                            return Err(());
                        }
                    }
                }
                Mutable::Mutate(mutate) => {
                    if let Some(proof_uri) = &mutate.uri {
                        if self_uri != proof_uri {
                            return Err(());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
