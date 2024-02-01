use super::arguments::Arguments;
use crate::{ability::command::Command, delegation::Delegatable, promise::Promise};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

// NOTE This one is primarily for enabling delegationd recipets

// FIXME
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Generic<Args> {
    pub cmd: String,
    pub args: Args, // FIXME Does this have specific fields?
}

pub type Resolved = Generic<Arguments>;
pub type Builder = Generic<Option<Arguments>>;
pub type Promised = Generic<Promise<Arguments>>;

impl<Args> Command for Generic<Args> {
    const COMMAND: &'static str = "ucan/proxy";
}

impl Delegatable for Resolved {
    type Builder = Builder;
}

impl From<Resolved> for Builder {
    fn from(resolved: Resolved) -> Builder {
        Builder {
            cmd: resolved.cmd,
            args: Some(resolved.args),
        }
    }
}

impl TryFrom<Builder> for Resolved {
    type Error = (); // FIXME

    fn try_from(b: Builder) -> Result<Self, Self::Error> {
        Ok(Resolved {
            cmd: b.cmd,
            args: b.args.ok_or(())?,
        })
    }
}

impl From<Builder> for Arguments {
    fn from(b: Builder) -> Arguments {
        let mut args = b.args.unwrap_or_default();
        args.insert("cmd".into(), Ipld::String(b.cmd));
        args
    }
}

// // FIXME hmmm
// #[derive(Debug, Clone, PartialEq)]
// pub struct ProxyExecuteBuilder {
//     pub command: Option<String>,
//     pub args: BTreeMap<String, Ipld>,
// }
//
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
