use crate::{
    ability::traits::{Command, Delegatable},
    signature,
};
use libipld_core::ipld::Ipld;
use std::{collections::BTreeMap, fmt::Debug};

pub mod payload;
use payload::Payload;

pub type Receipt<B> = signature::Envelope<Payload<B>>;

// FIXME show piping ability

// FIXME
#[derive(Debug, Clone, PartialEq)]
pub struct ProxyExecute {
    pub command: String,
    pub args: BTreeMap<String, Ipld>,
}

impl Delegatable for ProxyExecute {
    type Builder = ProxyExecuteBuilder;
}

// FIXME hmmm
#[derive(Debug, Clone, PartialEq)]
pub struct ProxyExecuteBuilder {
    pub command: Option<String>,
    pub args: BTreeMap<String, Ipld>,
}

impl Command for ProxyExecute {
    const COMMAND: &'static str = "ucan/proxy";
}

impl From<ProxyExecute> for ProxyExecuteBuilder {
    fn from(proxy: ProxyExecute) -> Self {
        ProxyExecuteBuilder {
            command: Some(ProxyExecute::COMMAND.into()),
            args: proxy.args.clone(),
        }
    }
}

impl TryFrom<ProxyExecuteBuilder> for ProxyExecute {
    type Error = (); // FIXME

    fn try_from(ProxyExecuteBuilder { command, args }: ProxyExecuteBuilder) -> Result<Self, ()> {
        match command {
            None => Err(()),
            Some(command) => Ok(Self { command, args }),
        }
    }
}
