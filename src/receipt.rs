use crate::{
    ability::traits::{Buildable, Command, Runnable},
    delegation::{condition::Condition, Delegation},
    invocation::Invocation,
    signature,
    time::Timestamp,
};
use did_url::DID;
use libipld_core::{cid::Cid, ipld::Ipld, link::Link};
use std::{collections::BTreeMap, fmt::Debug};

pub type Receipt<B> = signature::Envelope<Payload<B>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<B: Runnable + Debug> {
    pub issuer: DID,
    pub ran: Cid,
    pub out: Result<B::Output, BTreeMap<String, Ipld>>,

    pub proofs: Vec<Cid>,
    pub metadata: BTreeMap<String, Ipld>,
    pub issued_at: Option<Timestamp>,
}

impl<B: Runnable + Debug> signature::Capsule for Payload<B> {
    const TAG: &'static str = "ucan/r/1.0.0-rc.1"; // FIXME extract out version
}

// FIXME
#[derive(Debug, Clone, PartialEq)]
pub struct ProxyExecute {
    pub command: String,
    pub args: BTreeMap<String, Ipld>,
}

impl Buildable for ProxyExecute {
    type Builder = ProxyExecuteBuilder;

    fn to_builder(&self) -> Self::Builder {
        ProxyExecuteBuilder {
            command: Some(self.command.clone()),
            args: self.args.clone(),
        }
    }

    fn try_build(ProxyExecuteBuilder { command, args }: Self::Builder) -> Result<Box<Self>, ()> {
        match command {
            None => Err(()),
            Some(command) => Ok(Box::new(Self { command, args })),
        }
    }
}

// FIXME hmmm
#[derive(Debug, Clone, PartialEq)]
pub struct ProxyExecuteBuilder {
    pub command: Option<String>,
    pub args: BTreeMap<String, Ipld>,
}

impl Command for ProxyExecuteBuilder {
    fn command(&self) -> &'static str {
        "ucan/proxy" // FIXME check spec
    }
}
