//! The ability to send messages

use crate::{
    ability::{arguments, command::Command},
    delegation::Delegable,
    invocation::promise,
    ipld,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
// use url::Url;
use crate::url;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The executable/dispatchable variant of the `msg/send` ability.
///
/// # Lifecycle
///
/// The hierarchy of message abilities is as follows:
///
/// ```mermaid
/// flowchart LR
///     subgraph Delegations
///       top("*")
///
///       any("msg/*")
///
///       subgraph Invokable
///         send("msg/send")
///       end
///     end
///
///     sendpromise("msg::send::Promised")
///     sendrun("msg::send::Ready")
///
///     top --> any
///     any --> send -.->|invoke| sendpromise -.->|resolve| sendrun -.-> exe{{execute}}
///
///     style sendrun stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ready {
    /// The recipient of the message
    pub to: url::Newtype,

    /// The sender address of the message
    ///
    /// This *may* be a URL (such as an email address).
    /// If provided, the `subject` must have the right to send from this address.
    pub from: url::Newtype,

    /// The main body of the message
    pub message: String,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The delegatable variant of the `msg/send` ability.
///
/// # Delegation Hierarchy
///
/// The hierarchy of message abilities is as follows:
///
/// ```mermaid
/// flowchart LR
///     top("*")
///
///     subgraph Message Abilities
///       any("msg/*")
///
///       subgraph Invokable
///         send("msg/send")
///       end
///     end
///
///     sendrun{{"invoke"}}
///
///     top --> any
///     any --> send -.-> sendrun
///
///     style send stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Builder {
    /// The recipient of the message
    pub to: Option<url::Newtype>,

    /// The sender address of the message
    ///
    /// This *may* be a URL (such as an email address).
    /// If provided, the `subject` must have the right to send from this address.
    pub from: Option<url::Newtype>,

    /// The main body of the message
    pub message: Option<String>,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The invoked variant of the `msg/send` ability
///
/// This variant may be linked to other invoked abilities by [`Promise`][crate::invocation::Promise]s.
///
/// # Lifecycle
///
/// The hierarchy of message abilities is as follows:
///
/// ```mermaid
/// flowchart LR
///     subgraph Delegations
///       top("*")
///
///       any("msg/*")
///
///       subgraph Invokable
///         send("msg/send")
///       end
///     end
///
///     sendpromise("msg::send::Promised")
///     sendrun("msg::send::Ready")
///
///     top --> any
///     any --> send -.->|invoke| sendpromise -.->|resolve| sendrun -.-> exe{{execute}}
///
///     style sendpromise stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Promised {
    /// The recipient of the message
    pub to: promise::Resolves<url::Newtype>,

    /// The sender address of the message
    ///
    /// This *may* be a URL (such as an email address).
    /// If provided, the `subject` must have the right to send from this address.
    pub from: promise::Resolves<url::Newtype>,

    /// The main body of the message
    pub message: promise::Resolves<String>,
}

impl Delegable for Ready {
    type Builder = Builder;
}

impl promise::Resolvable for Ready {
    type Promised = Promised;
}

impl TryFrom<arguments::Named<ipld::Promised>> for Promised {
    type Error = ();

    fn try_from(args: arguments::Named<ipld::Promised>) -> Result<Promised, Self::Error> {
        let mut to = None;
        let mut from = None;
        let mut message = None;

        for (key, prom) in args.0 {
            match key.as_str() {
                "to" => match Ipld::try_from(prom) {
                    Ok(Ipld::String(s)) => {
                        to = Some(promise::Resolves::from(Ok(
                            url::Newtype::parse(s.as_str()).map_err(|_| ())?
                        )));
                    }
                    Err(pending) => to = Some(pending.into()),
                    _ => return Err(()),
                },
                "from" => match Ipld::try_from(prom) {
                    Ok(Ipld::String(s)) => {
                        from = Some(promise::Resolves::from(Ok(
                            url::Newtype::parse(s.as_str()).map_err(|_| ())?
                        )));
                    }
                    Err(pending) => from = Some(pending.into()),
                    _ => return Err(()),
                },
                "message" => match Ipld::try_from(prom) {
                    Ok(Ipld::String(s)) => message = Some(promise::Resolves::from(Ok(s))),
                    Err(pending) => to = Some(pending.into()),
                    _ => return Err(()),
                },
                _ => return Err(()),
            }
        }

        Ok(Promised {
            to: to.ok_or(())?,
            from: from.ok_or(())?,
            message: message.ok_or(())?,
        })
    }
}

impl From<Builder> for arguments::Named<Ipld> {
    fn from(b: Builder) -> Self {
        let mut btree = BTreeMap::new();
        b.to.map(|to| btree.insert("to".into(), to.to_string().into()));
        b.from
            .map(|from| btree.insert("from".into(), from.to_string().into()));
        b.message
            .map(|msg| btree.insert("message".into(), msg.into()));

        arguments::Named(btree)
    }
}

impl From<Promised> for arguments::Named<Ipld> {
    fn from(p: Promised) -> Self {
        arguments::Named::from_iter([
            ("to".into(), p.to.into()),
            ("from".into(), p.from.into()),
            ("message".into(), p.message.into()),
        ])
    }
}

impl From<Promised> for Builder {
    fn from(p: Promised) -> Self {
        Builder {
            to: p.to.into(),
            from: p.from.into(),
            message: p.message.into(),
        }
    }
}

const COMMAND: &'static str = "msg/send";

impl Command for Ready {
    const COMMAND: &'static str = COMMAND;
}

impl Command for Builder {
    const COMMAND: &'static str = COMMAND;
}

impl Command for Promised {
    const COMMAND: &'static str = COMMAND;
}

impl Checkable for Builder {
    type Hierarchy = Parentful<Builder>;
}

impl CheckSame for Builder {
    type Error = (); // FIXME better error

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        self.to.check_same(&proof.to).map_err(|_| ())?;
        self.from.check_same(&proof.from).map_err(|_| ())?;
        self.message.check_same(&proof.message).map_err(|_| ())
    }
}

impl CheckParents for Builder {
    type Parents = super::Any;
    type ParentError = <super::Any as CheckSame>::Error;

    fn check_parent(&self, proof: &Self::Parents) -> Result<(), Self::ParentError> {
        self.from.check_same(&proof.from).map_err(|_| ())
    }
}

impl TryFrom<arguments::Named<Ipld>> for Builder {
    type Error = ();

    fn try_from(args: arguments::Named<Ipld>) -> Result<Builder, Self::Error> {
        let mut to = None;
        let mut from = None;
        let mut message = None;

        for (key, ipld) in args.0 {
            match key.as_str() {
                "to" => {
                    // FIXME extract this common pattern
                    if let Ipld::String(s) = ipld {
                        to = Some(url::Newtype::parse(s.as_str()).map_err(|_| ())?);
                    } else {
                        return Err(());
                    }
                }
                "from" => {
                    if let Ipld::String(s) = ipld {
                        from = Some(url::Newtype::parse(s.as_str()).map_err(|_| ())?);
                    } else {
                        return Err(());
                    }
                }
                "message" => {
                    if let Ipld::String(s) = ipld {
                        message = Some(s);
                    } else {
                        return Err(());
                    }
                }
                _ => return Err(()),
            }
        }

        Ok(Builder { to, from, message })
    }
}

impl From<Ready> for Builder {
    fn from(resolved: Ready) -> Self {
        Builder {
            to: resolved.to.into(),
            from: resolved.from.into(),
            message: resolved.message.into(),
        }
    }
}

impl From<Ready> for Promised {
    fn from(r: Ready) -> Self {
        Promised {
            to: promise::Resolves::from(Ok(r.to)),
            from: promise::Resolves::from(Ok(r.from)),
            message: promise::Resolves::from(Ok(r.message)),
        }
    }
}

impl TryFrom<Promised> for Ready {
    type Error = Promised;

    fn try_from(p: Promised) -> Result<Ready, Promised> {
        match promise::Resolves::try_resolve_3(p.to, p.from, p.message) {
            Ok((to, from, message)) => Ok(Ready { to, from, message }),
            Err((to, from, message)) => Err(Promised { to, from, message }),
        }
    }
}

impl TryFrom<Builder> for Ready {
    type Error = Builder;

    fn try_from(b: Builder) -> Result<Self, Builder> {
        // Entirely by refernce
        if b.to.is_none() || b.from.is_none() || b.message.is_none() {
            return Err(b);
        }

        // Moves, and unwrap because we checked above instead of 2 clones per line
        Ok(Ready {
            to: b.to.unwrap(),
            from: b.from.unwrap(),
            message: b.message.unwrap(),
        })
    }
}

impl From<Promised> for arguments::Named<ipld::Promised> {
    fn from(p: Promised) -> Self {
        arguments::Named::from_iter([
            ("to".into(), p.to.into()),
            ("from".into(), p.from.into()),
            ("message".into(), p.message.into()),
        ])
    }
}
