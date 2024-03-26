//! This is an ability for revoking [`Delegation`][crate::delegation::Delegation]s by their [`Cid`].
//!
//! For more, see the [UCAN Revocation spec](https://github.com/ucan-wg/revocation).

use crate::{
    ability::{arguments, command::Command},
    invocation::promise,
    ipld,
};
use libipld_core::{cid::Cid, ipld::Ipld};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// The fully resolved variant: ready to execute.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Revoke {
    /// The UCAN to revoke
    pub ucan: Cid,
    // FIXME pub witness
}

impl From<Revoke> for arguments::Named<Ipld> {
    fn from(revoke: Revoke) -> Self {
        arguments::Named::from_iter([("ucan".to_string(), Ipld::Link(revoke.ucan).into())])
    }
}

const COMMAND: &'static str = "/ucan/revoke";

impl Command for Revoke {
    const COMMAND: &'static str = COMMAND;
}
impl Command for PromisedRevoke {
    const COMMAND: &'static str = COMMAND;
}

impl TryFrom<arguments::Named<Ipld>> for Revoke {
    type Error = ();

    fn try_from(arguments: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        let ipld: Ipld = arguments.get("ucan").ok_or(())?.clone();
        let nt: ipld::cid::Newtype = ipld.try_into().map_err(|_| ())?;

        Ok(Revoke { ucan: nt.cid })
    }
}

impl promise::Resolvable for Revoke {
    type Promised = PromisedRevoke;
}

impl From<PromisedRevoke> for arguments::Named<ipld::Promised> {
    fn from(promised: PromisedRevoke) -> Self {
        arguments::Named::from_iter([("ucan".into(), Ipld::from(promised.ucan).into())])
    }
}

/// A variant where arguments may be [`Promise`][crate::invocation::promise]s.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromisedRevoke {
    pub ucan: promise::Any<Cid>,
}

impl TryFrom<arguments::Named<ipld::Promised>> for PromisedRevoke {
    type Error = ();

    fn try_from(arguments: arguments::Named<ipld::Promised>) -> Result<Self, Self::Error> {
        let mut ucan = None;

        for (k, prom) in arguments {
            match k.as_str() {
                "ucan" => match Ipld::try_from(prom) {
                    Ok(Ipld::Link(cid)) => {
                        ucan = Some(promise::Any::Resolved(cid));
                    }
                    Err(pending) => ucan = Some(pending.into()),
                    _ => return Err(()),
                },
                _ => (),
            }
        }

        Ok(PromisedRevoke {
            ucan: ucan.ok_or(())?,
        })
    }
}

impl From<Revoke> for PromisedRevoke {
    fn from(r: Revoke) -> PromisedRevoke {
        PromisedRevoke {
            ucan: promise::Any::Resolved(r.ucan),
        }
    }
}

impl From<PromisedRevoke> for arguments::Named<Ipld> {
    fn from(p: PromisedRevoke) -> arguments::Named<Ipld> {
        arguments::Named::from_iter([("ucan".into(), p.ucan.into())])
    }
}

impl TryFrom<PromisedRevoke> for Revoke {
    type Error = ();

    fn try_from(p: PromisedRevoke) -> Result<Self, Self::Error> {
        Ok(Revoke {
            ucan: p.ucan.try_resolve().map_err(|_| ())?,
        })
    }
}
