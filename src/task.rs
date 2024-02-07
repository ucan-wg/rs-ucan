//! Task indices for [`Receipt`][crate::receipt::Receipt] reverse lookup

use crate::{ability::arguments, did::Did, nonce::Nonce};
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
use std::fmt::Debug;

const SHA2_256: u64 = 0x12;

/// The fields required to uniquely identify a [`Task`], potentially across multiple executors.
///
/// This struct should not be used directly, but rather through a [`From`] instance
/// on the type. In particular, the `nonce` field should be constant for all of the same type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    /// The `subject`: root issuer, and arbiter of the semantics/namespace
    pub sub: Did,

    /// A unique identifier for the particular task run
    ///
    /// This is an [`Option`] because not all task types require a nonce.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<Nonce>,

    /// The command identifier
    pub cmd: String,

    /// The arguments to the command
    pub args: arguments::Named,
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

/// The unique identifier for a [`Task`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
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

impl From<Task> for Id {
    fn from(task: Task) -> Id {
        Id {
            cid: Cid::from(task),
        }
    }
}
