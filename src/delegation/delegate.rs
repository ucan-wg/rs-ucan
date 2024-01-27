use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Delegate<T> {
    #[serde(rename = "ucan/*")]
    Any,
    Specific(T),
}

impl<T: TryFrom<Ipld> + serde::de::DeserializeOwned> TryFrom<Ipld> for Delegate<T> {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, ()> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}
