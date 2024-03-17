//! The ability to receive messages

use crate::{
    ability::{arguments, command::Command},
    invocation::promise,
    ipld, url,
};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};

#[cfg(feature = "test_utils")]
use proptest_derive::Arbitrary;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The ability to receive messages
///
/// This ability is used to receive messages from other actors.
///
/// # Delegation Hierarchy
///
/// The hierarchy of message abilities is as follows:
///
/// ```mermaid
/// flowchart TB
///     top("*")
///
///     subgraph Message Abilities
///       any("msg/*")
///
///       subgraph Invokable
///         rec("msg/receive")
///       end
///     end
///
///     recrun{{"invoke"}}
///
///     top --> any
///     any --> rec -.-> recrun
///
///     style rec stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "test_utils", derive(Arbitrary))]
#[serde(deny_unknown_fields)]
pub struct Receive {
    /// An *optional* URL (e.g. email, DID, socket) to receive messages from.
    /// This assumes that the `subject` has the authority to issue such a capability.
    pub from: Option<url::Newtype>,
}

const COMMAND: &'static str = "/msg/receive";

impl Command for Receive {
    const COMMAND: &'static str = COMMAND;
}

impl Command for PromisedReceive {
    const COMMAND: &'static str = COMMAND;
}

impl TryFrom<arguments::Named<Ipld>> for Receive {
    type Error = ();

    fn try_from(arguments: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        let mut from = None;

        for (key, ipld) in arguments {
            match key.as_str() {
                "from" => {
                    from = Some(url::Newtype::try_from(ipld).map_err(|_| ())?);
                }
                _ => return Err(()),
            }
        }

        Ok(Receive { from })
    }
}

impl From<Receive> for arguments::Named<Ipld> {
    fn from(receive: Receive) -> Self {
        let mut args = arguments::Named::new();

        if let Some(from) = receive.from {
            args.insert("from".into(), from.into());
        }

        args
    }
}

impl From<Receive> for Ipld {
    fn from(receive: Receive) -> Self {
        arguments::Named::<Ipld>::from(receive).into()
    }
}

impl TryFrom<Ipld> for Receive {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::Map(map) = ipld {
            arguments::Named::<Ipld>(map).try_into().map_err(|_| ())
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromisedReceive {
    pub from: Option<promise::Any<url::Newtype>>,
}

impl promise::Resolvable for Receive {
    type Promised = PromisedReceive;
}

impl TryFrom<arguments::Named<ipld::Promised>> for PromisedReceive {
    type Error = ();

    fn try_from(arguments: arguments::Named<ipld::Promised>) -> Result<Self, Self::Error> {
        let mut from = None;

        for (key, prom) in arguments {
            match key.as_str() {
                "from" => match Ipld::try_from(prom) {
                    Ok(Ipld::String(s)) => {
                        from = Some(promise::Any::Resolved(
                            url::Newtype::parse(s.as_str()).map_err(|_| ())?,
                        ));
                    }
                    Err(pending) => from = Some(pending.into()),
                    _ => return Err(()),
                },
                _ => return Err(()),
            }
        }

        Ok(PromisedReceive { from })
    }
}

impl From<PromisedReceive> for arguments::Named<ipld::Promised> {
    fn from(promised: PromisedReceive) -> Self {
        let mut args = arguments::Named::new();

        if let Some(from) = promised.from {
            let _ = from.to_promised_ipld().with_resolved(|ipld| {
                args.insert("from".into(), ipld.into());
            });
        }

        args
    }
}
