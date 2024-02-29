//! This is an ability for revoking [`Delegation`][crate::delegation::Delegation]s by their [`Cid`].
//!
//! For more, see the [UCAN Revocation spec](https://github.com/ucan-wg/revocation).

use crate::{
    ability::{arguments, command::Command},
    //     delegation::Delegable,
    invocation::promise,
    ipld,
    // proof::{error::OptionalFieldError, parentless::NoParents, same::CheckSame},
};
use libipld_core::{cid::Cid, ipld::Ipld};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// The fully resolved variant: ready to execute.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ready {
    /// The UCAN to revoke
    pub ucan: Cid,
}

const COMMAND: &'static str = "/ucan/revoke";

impl Command for Ready {
    const COMMAND: &'static str = COMMAND;
}
impl Command for Promised {
    const COMMAND: &'static str = COMMAND;
}

impl TryFrom<arguments::Named<Ipld>> for Ready {
    type Error = ();

    fn try_from(arguments: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        let ipld: Ipld = arguments.get("ucan").ok_or(())?.clone();
        let nt: ipld::cid::Newtype = ipld.try_into().map_err(|_| ())?;

        Ok(Ready { ucan: nt.cid })
    }
}

impl promise::Resolvable for Ready {
    type Promised = Promised;
}

impl From<Promised> for arguments::Named<ipld::Promised> {
    fn from(promised: Promised) -> Self {
        arguments::Named::from_iter([("ucan".into(), promised.ucan.into())])
    }
}

/// A variant where arguments may be [`Promise`][crate::invocation::promise]s.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Promised {
    pub ucan: promise::Resolves<Cid>,
}

impl TryFrom<arguments::Named<ipld::Promised>> for Promised {
    type Error = ();

    fn try_from(arguments: arguments::Named<ipld::Promised>) -> Result<Self, Self::Error> {
        let mut ucan = None;

        for (k, prom) in arguments {
            match k.as_str() {
                "ucan" => match Ipld::try_from(prom) {
                    Ok(Ipld::Link(cid)) => {
                        ucan = Some(promise::Resolves::new(cid));
                    }
                    Err(pending) => ucan = Some(pending.into()),
                    _ => return Err(()),
                },
                _ => (),
            }
        }

        Ok(Promised {
            ucan: ucan.ok_or(())?,
        })
    }
}

impl From<Ready> for Promised {
    fn from(r: Ready) -> Promised {
        Promised {
            ucan: Ok(r.ucan).into(),
        }
    }
}

impl From<Promised> for arguments::Named<Ipld> {
    fn from(p: Promised) -> arguments::Named<Ipld> {
        arguments::Named::from_iter([("ucan".into(), p.ucan.into())])
    }
}

impl TryFrom<Promised> for Ready {
    type Error = ();

    fn try_from(p: Promised) -> Result<Self, Self::Error> {
        Ok(Ready {
            ucan: p.ucan.try_resolve().map_err(|_| ())?,
        })
    }
}
