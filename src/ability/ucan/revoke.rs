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

const COMMAND: &'static str = "ucan/revoke";

impl Command for Ready {
    const COMMAND: &'static str = COMMAND;
}

// impl Command for Builder {
//     const COMMAND: &'static str = COMMAND;
// }

impl Command for Promised {
    const COMMAND: &'static str = COMMAND;
}

// impl Delegable for Ready {
//     type Builder = Builder;
// }

impl TryFrom<arguments::Named<Ipld>> for Ready {
    type Error = ();

    fn try_from(arguments: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        let ipld: Ipld = arguments.get("ucan").ok_or(())?.clone();
        let nt: ipld::cid::Newtype = ipld.try_into().map_err(|_| ())?;

        Ok(Ready { ucan: nt.cid })
    }
}

// impl TryFrom<arguments::Named<Ipld>> for Builder {
//     type Error = ();
//
//     fn try_from(arguments: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
//         if let Some(ipld) = arguments.get("ucan") {
//             let nt: ipld::cid::Newtype = ipld.try_into().map_err(|_| ())?;
//             Ok(Builder { ucan: Some(nt.cid) })
//         } else {
//             Ok(Builder { ucan: None })
//         }
//     }
// }

// impl From<Promised> for Builder {
//     fn from(promised: Promised) -> Self {
//         Builder {
//             ucan: promised.ucan.try_resolve().ok(),
//         }
//     }
// }

impl promise::Resolvable for Ready {
    type Promised = Promised;
}

impl From<Promised> for arguments::Named<ipld::Promised> {
    fn from(promised: Promised) -> Self {
        arguments::Named::from_iter([("ucan".into(), promised.ucan.into())])
    }
}

// /// A variant with some fields waiting to be set.
// #[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
// pub struct Builder {
//     pub ucan: Option<Cid>,
// }
//
// impl NoParents for Builder {}
//
// impl CheckSame for Builder {
//     type Error = OptionalFieldError;
//
//     fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
//         self.ucan.check_same(&proof.ucan)
//     }
// }

// impl From<Ready> for Builder {
//     fn from(resolved: Ready) -> Builder {
//         Builder {
//             ucan: Some(resolved.ucan),
//         }
//     }
// }

// impl TryFrom<Builder> for Ready {
//     type Error = ();
//
//     fn try_from(b: Builder) -> Result<Self, Self::Error> {
//         Ok(Ready {
//             ucan: b.ucan.ok_or(())?,
//         })
//     }
// }
//
// impl From<Builder> for arguments::Named<Ipld> {
//     fn from(b: Builder) -> arguments::Named<Ipld> {
//         let mut btree = BTreeMap::new();
//         if let Some(cid) = b.ucan {
//             btree.insert("ucan".into(), cid.into());
//         }
//         arguments::Named(btree)
//     }
// }

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
