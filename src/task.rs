use crate::{did::Did, nonce::Nonce};
use libipld_cbor::DagCborCodec;
use libipld_core::{
    cid::{Cid, CidGeneric},
    codec::Encode,
    error::SerdeError,
    ipld::Ipld,
    multihash::MultihashGeneric,
    serde as ipld_serde,
};
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

const SHA2_256: u64 = 0x12;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub: Option<Did>, // Is this optional? May as well make it so for now!
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<Nonce>,

    pub cmd: String,
    pub args: BTreeMap<String, Ipld>,
}

impl From<Task> for Id {
    fn from(task: Task) -> Id {
        Id {
            cid: Cid::from(task),
        }
    }
}

impl TryFrom<Ipld> for Task {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl From<Task> for Ipld {
    fn from(task: Task) -> Self {
        task.into()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Id {
    pub cid: Cid,
}

impl TryFrom<Ipld> for Id {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl From<Id> for Ipld {
    fn from(id: Id) -> Self {
        id.cid.into()
    }
}

// FIXME move to differet module
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DefaultTrue(pub bool);

impl From<DefaultTrue> for bool {
    fn from(def_true: DefaultTrue) -> Self {
        def_true.0
    }
}

impl From<bool> for DefaultTrue {
    fn from(b: bool) -> Self {
        DefaultTrue(b)
    }
}

impl Default for DefaultTrue {
    fn default() -> Self {
        DefaultTrue(true)
    }
}

impl From<Task> for Cid {
    fn from(task: Task) -> Cid {
        let mut buffer = vec![];
        let ipld: Ipld = task.into();
        ipld.encode(DagCborCodec, &mut buffer)
            .expect("DagCborCodec to encode any arbitrary `Ipld`");
        CidGeneric::new_v1(
            DagCborCodec.into(),
            MultihashGeneric::wrap(SHA2_256, buffer.as_slice())
                .expect("DagCborCodec + Sha2_256 should always successfully encode Ipld to a Cid"),
        )
    }
}
