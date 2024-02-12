use crate::{
    ability::{arguments, command::Command},
    delegation::Delegable,
    invocation::Promise,
};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

// NOTE This one is primarily for enabling delegationd recipets

// FIXME can this *only* be a builder?
// NOTE UNLIKE the dynamic ability, this has cmd as an argument *at runtime*
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Generic<Args> {
    pub cmd: String,
    pub args: Args, // FIXME Does this have specific fields?
                    // FIXME should args just be a CID
}

pub type Ready = Generic<arguments::Named>;
pub type Builder = Generic<Option<arguments::Named>>;
pub type Promised = Generic<Promise<arguments::Named>>;

impl<Args> Command for Generic<Args> {
    const COMMAND: &'static str = "ucan/proxy";
}

impl Delegable for Ready {
    type Builder = Builder;
}

impl From<Ready> for Builder {
    fn from(resolved: Ready) -> Builder {
        Builder {
            cmd: resolved.cmd,
            args: Some(resolved.args),
        }
    }
}

impl TryFrom<Builder> for Ready {
    type Error = (); // FIXME

    fn try_from(b: Builder) -> Result<Self, Self::Error> {
        Ok(Ready {
            cmd: b.cmd,
            args: b.args.ok_or(())?,
        })
    }
}

impl From<Builder> for arguments::Named {
    fn from(b: Builder) -> arguments::Named {
        let mut args = b.args.unwrap_or_default();
        args.insert("cmd".into(), Ipld::String(b.cmd));
        args
    }
}
