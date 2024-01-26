use cid::Cid;
use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};
use std::fmt::Debug;

/// A [`Promise`] is a way to defer the presence of a value to the result of some [`Invocation`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)] // FIXME check that this is right, also
pub enum Promise {
    PromiseAny {
        #[serde(rename = "ucan/*")] // FIXME test to make sure that this is right?
        await_any: Cid,
    },
    PromiseOk {
        #[serde(rename = "ucan/ok")]
        await_ok: Cid,
    },
    PromiseErr {
        #[serde(rename = "ucan/err")]
        await_err: Cid,
    },
}

impl TryFrom<Ipld> for Promise {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}
