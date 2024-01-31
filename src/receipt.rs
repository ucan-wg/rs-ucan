use crate::signature;
use libipld_core::ipld::Ipld;
use payload::Payload;
use std::{collections::BTreeMap, fmt::Debug};

pub mod payload;
pub mod runnable;

pub type Receipt<T> = signature::Envelope<Payload<T>>;

// FIXME
#[derive(Debug, Clone, PartialEq)]
pub struct ProxyExecute {
    pub cmd: String,
    pub args: BTreeMap<String, Ipld>,
}

// impl Delegatable for ProxyExecute {
//     type Builder = ProxyExecuteBuilder;
// }
//
// // FIXME hmmm
// #[derive(Debug, Clone, PartialEq)]
// pub struct ProxyExecuteBuilder {
//     pub command: Option<String>,
//     pub args: BTreeMap<String, Ipld>,
// }
//
// impl Command for ProxyExecute {
//     const COMMAND: &'static str = "ucan/proxy";
// }
//
// impl From<ProxyExecute> for ProxyExecuteBuilder {
//     fn from(proxy: ProxyExecute) -> Self {
//         ProxyExecuteBuilder {
//             command: Some(ProxyExecute::COMMAND.into()),
//             args: proxy.args.clone(),
//         }
//     }
// }
//
// impl TryFrom<ProxyExecuteBuilder> for ProxyExecute {
//     type Error = (); // FIXME
//
//     fn try_from(ProxyExecuteBuilder { command, args }: ProxyExecuteBuilder) -> Result<Self, ()> {
//         match command {
//             None => Err(()),
//             Some(command) => Ok(Self { command, args }),
//         }
//     }
// }
