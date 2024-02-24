//! The ability to send messages

use crate::{
    ability::{arguments, command::Command},
    invocation::promise,
    ipld, url,
};
use libipld_core::{error::SerdeError, ipld::Ipld};
use serde::{Deserialize, Serialize};

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

// impl Delegable for Ready {
//     type Builder = Builder;
// }

impl promise::Resolvable for Ready {
    type Promised = Promised;
}

impl TryFrom<arguments::Named<Ipld>> for Ready {
    type Error = ();

    fn try_from(named: arguments::Named<Ipld>) -> Result<Ready, Self::Error> {
        let mut to = None;
        let mut from = None;
        let mut message = None;

        for (key, value) in named.0 {
            match key.as_str() {
                "to" => match Ipld::try_from(value) {
                    Ok(Ipld::String(s)) => {
                        to = Some(url::Newtype::parse(s.as_str()).map_err(|_| ())?)
                    }
                    _ => return Err(()),
                },
                "from" => match Ipld::try_from(value) {
                    Ok(Ipld::String(s)) => {
                        from = Some(url::Newtype::parse(s.as_str()).map_err(|_| ())?)
                    }
                    _ => return Err(()),
                },
                "message" => match Ipld::try_from(value) {
                    Ok(Ipld::String(s)) => message = Some(s),
                    _ => return Err(()),
                },
                _ => return Err(()),
            }
        }

        Ok(Ready {
            to: to.ok_or(())?,
            from: from.ok_or(())?,
            message: message.ok_or(())?,
        })
    }
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

impl From<Promised> for arguments::Named<Ipld> {
    fn from(p: Promised) -> Self {
        arguments::Named::from_iter([
            ("to".into(), p.to.into()),
            ("from".into(), p.from.into()),
            ("message".into(), p.message.into()),
        ])
    }
}

const COMMAND: &'static str = "msg/send";

impl Command for Ready {
    const COMMAND: &'static str = COMMAND;
}

impl Command for Promised {
    const COMMAND: &'static str = COMMAND;
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

impl From<Promised> for arguments::Named<ipld::Promised> {
    fn from(p: Promised) -> Self {
        arguments::Named::from_iter([
            ("to".into(), p.to.into()),
            ("from".into(), p.from.into()),
            ("message".into(), p.message.into()),
        ])
    }
}
