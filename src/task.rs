//! Task indices for [`Receipt`][crate::receipt::Receipt] reverse lookup.

mod id;
pub use id::Id;

use crate::{ability::arguments, crypto::Nonce, did};
use libipld_cbor::DagCborCodec;
use libipld_core::{
    cid::{Cid, CidGeneric},
    codec::Encode,
    error::SerdeError,
    ipld::Ipld,
    multihash::{Code, MultihashGeneric},
    serde as ipld_serde,
};
use serde_derive::{Deserialize, Serialize};
use std::fmt::Debug;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

/// The fields required to uniquely identify a [`Task`], potentially across multiple executors.
///
/// This struct should not be used directly, but rather through a [`From`] instance
/// on the type. In particular, the `nonce` field should be constant for all of the same type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    /// The `subject`: root issuer, and arbiter of the semantics/namespace.
    pub sub: did::Newtype,

    /// A unique identifier for the particular task run.
    ///
    /// This is an [`Option`] because not all task types require a nonce.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<Nonce>,

    /// The command identifier.
    pub cmd: String,

    /// The arguments to the command.
    pub args: arguments::Named<Ipld>,
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
            MultihashGeneric::wrap(Code::Sha2_256.into(), buffer.as_slice())
                .expect("DagCborCodec + Sha2_256 should always successfully encode Ipld to a Cid"),
        )
    }
}

impl From<Task> for Id {
    fn from(task: Task) -> Id {
        Id {
            cid: Cid::from(task),
        }
    }
}

#[cfg(feature = "test_utils")]
impl Arbitrary for Task {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            any::<did::Newtype>(),
            any::<Option<Nonce>>(),
            any::<String>(),
            any::<arguments::Named<Ipld>>(),
        )
            .prop_map(|(sub, nonce, cmd, args)| Task {
                sub,
                nonce,
                cmd,
                args,
            })
            .boxed()
    }
}
