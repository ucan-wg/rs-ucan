use crate::{did::Did, nonce::Nonce, prove::TryProve};
use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

pub trait Command {
    const COMMAND: &'static str;
}

// FIXME Delegable and make it proven?
pub trait Delegatable: Sized {
    type Builder: Debug + TryInto<Self> + From<Self>;
}

pub trait Resolvable: Sized {
    type Awaiting: Debug + TryInto<Self> + From<Self>;
}

pub trait Runnable {
    type Output: Debug;
    fn task_id(self, subject: &Did, nonce: &Nonce) -> String;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynJs {
    pub cmd: String,
    pub args: BTreeMap<String, Ipld>,
}

impl Delegatable for DynJs {
    type Builder = Self;
}

impl Resolvable for DynJs {
    type Awaiting = Self;
}

impl From<DynJs> for Ipld {
    fn from(js: DynJs) -> Self {
        js.into()
    }
}

impl TryFrom<Ipld> for DynJs {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}
