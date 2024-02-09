//! UCAN [Revocations](https://github.com/ucan-wg/revocation)

use crate::{
    ability::{arguments, command::Command},
    delegation::Delegatable,
    invocation::{promise, Resolvable},
    proof::{parentless::NoParents, same::CheckSame},
};
use libipld_core::{cid::Cid, ipld::Ipld};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

/// An ability for revoking previously issued UCANs by [`Cid`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Generic<Arg> {
    // FIXME check spec
    /// The UCAN to revoke
    pub ucan: Arg,
}

impl<Arg> Command for Generic<Arg> {
    const COMMAND: &'static str = "ucan/revoke";
}

/// The fully resolved variant: ready to execute.
pub type Ready = Generic<Cid>;

impl NoParents for Ready {}

impl Delegatable for Ready {
    type Builder = Builder;
}

impl Resolvable for Ready {
    type Promised = Promised;

    fn try_resolve(promised: Self::Promised) -> Result<Self, Self::Promised> {
        match promised.ucan.try_resolve() {
            Ok(ucan) => Ok(Ready { ucan }),
            Err(ucan) => Err(Promised { ucan }),
        }
    }
}

/// A variant with some fields waiting to be set.
pub type Builder = Generic<Option<Cid>>;

impl CheckSame for Builder {
    type Error = (); // FIXME

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        self.ucan.check_same(&proof.ucan).map_err(|_| ())
    }
}

impl From<Ready> for Builder {
    fn from(resolved: Ready) -> Builder {
        Builder {
            ucan: Some(resolved.ucan),
        }
    }
}

impl TryFrom<Builder> for Ready {
    type Error = ();

    fn try_from(b: Builder) -> Result<Self, Self::Error> {
        Ok(Ready {
            ucan: b.ucan.ok_or(())?,
        })
    }
}

impl From<Builder> for arguments::Named<Ipld> {
    fn from(b: Builder) -> arguments::Named<Ipld> {
        let mut btree = BTreeMap::new();
        if let Some(cid) = b.ucan {
            btree.insert("ucan".into(), cid.into());
        }
        arguments::Named(btree)
    }
}

/// A variant where arguments may be [`Promise`]s.
pub type Promised = Generic<promise::Resolves<Cid>>;

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
