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
    type Builder: TryInto<Self> + From<Self>;
}

pub trait Resolvable: Delegatable {
    type Awaiting: TryInto<Self> + From<Self> + Into<Self::Builder>;
}

pub trait Runnable {
    type Output;

    fn to_task(&self, subject: Did, nonce: Nonce) -> Task;

    fn to_task_id(&self, subject: Did, nonce: Nonce) -> TaskId {
        TaskId {
            cid: self.to_task(subject, nonce).into(),
        }
    }

    // fn lookup(id: TaskId>) -> Result<Self::Output, ()>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskId {
    cid: Cid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynJs {
    pub cmd: String,
    #[serde(default)]
    pub args: BTreeMap<String, Ipld>,

    #[serde(default)]
    pub serialize_nonce: DefaultTrue,
}

// FIXME move to differet module
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DefaultTrue(bool);

impl Default for DefaultTrue {
    fn default() -> Self {
        DefaultTrue(true)
    }
}

impl Delegatable for DynJs {
    type Builder = Self;
}

impl Resolvable for DynJs {
    type Awaiting = Self;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub: Option<Did>, // Is this optional? May as well make it so for now!
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<Nonce>,

    pub cmd: String,
    pub args: BTreeMap<String, Ipld>,
}

impl From<Task> for Ipld {
    fn from(task: Task) -> Ipld {
        task.into()
    }
}

impl From<Task> for Cid {
    fn from(task: Task) -> Cid {
        let mut buffer = vec![];
        let ipld: Ipld = task.into();
        ipld.encode(DagCborCodec, &mut buffer)
            .expect("DagCborCodec to encode any arbitrary `Ipld`");
        CidGeneric::new_v1(DagCborCodec.into(), Sha2_256.digest(buffer.as_slice()))
    }
}

// FIXME DynJs may need a hook for if the nonce should be included
impl Runnable for DynJs {
    type Output = Ipld;

    fn to_task(&self, subject: Did, nonce: Nonce) -> Task {
        Task {
            sub: Some(subject),
            nonce: if self.serialize_nonce == DefaultTrue(true) {
                Some(nonce)
            } else {
                None
            },
            cmd: self.cmd.clone(),
            args: self.args.clone(),
        }
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
