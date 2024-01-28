use crate::{did::Did, nonce::Nonce};
use libipld_cbor::DagCborCodec;
use libipld_core::{
    cid::{Cid, CidGeneric},
    codec::Encode,
    ipld::Ipld,
    multihash::{Code::Sha2_256, MultihashDigest},
    serde as ipld_serde,
};
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

pub trait Command {
    const COMMAND: &'static str;
}

// FIXME Delegable and make it proven?
pub trait Delegatable: Sized {
    type Builder: Debug + TryInto<Self> + From<Self>;
}

pub trait Resolvable: Delegatable {
    type Awaiting: Debug + TryInto<Self> + From<Self> + Into<Self::Builder>;
}

pub trait Runnable {
    type Output: Debug;
    fn task_id(self, subject: Did, nonce: Nonce) -> Cid;
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

impl Runnable for DynJs {
    type Output = Ipld;

    fn task_id(self, subject: Did, nonce: Nonce) -> Cid {
        let ipld: Ipld = BTreeMap::from_iter([
            ("sub".into(), subject.into()),
            ("do".into(), self.cmd.clone().into()),
            ("args".into(), self.cmd.clone().into()),
            ("nonce".into(), nonce.into()),
        ])
        .into();

        let mut encoded = vec![];
        ipld.encode(DagCborCodec, &mut encoded)
            .expect("should never fail if `encodable_as` is implemented correctly");

        let multihash = Sha2_256.digest(encoded.as_slice());
        CidGeneric::new_v1(DagCborCodec.into(), multihash)
    }
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
