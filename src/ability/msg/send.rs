//! The ability to send messages

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
///     sendrun("msg::send::Send")
///
///     top --> any
///     any --> send -.->|invoke| sendpromise -.->|resolve| sendrun -.-> exe{{execute}}
///
///     style sendrun stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "test_utils", derive(Arbitrary))]
#[serde(deny_unknown_fields)]
pub struct Send {
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

impl From<Send> for arguments::Named<Ipld> {
    fn from(send: Send) -> Self {
        arguments::Named::from_iter([
            ("to".to_string(), send.to.into()),
            ("from".to_string(), send.from.into()),
            ("message".to_string(), send.message.into()),
        ])
    }
}

impl From<Send> for Ipld {
    fn from(send: Send) -> Self {
        let args = arguments::Named::from(send);
        Ipld::Map(args.0)
    }
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
///     sendrun("msg::send::Send")
///
///     top --> any
///     any --> send -.->|invoke| sendpromise -.->|resolve| sendrun -.-> exe{{execute}}
///
///     style sendpromise stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromisedSend {
    /// The recipient of the message
    pub to: promise::Any<url::Newtype>,

    /// The sender address of the message
    ///
    /// This *may* be a URL (such as an email address).
    /// If provided, the `subject` must have the right to send from this address.
    pub from: promise::Any<url::Newtype>,

    /// The main body of the message
    pub message: promise::Any<String>,
}

impl promise::Resolvable for Send {
    type Promised = PromisedSend;
}

impl TryFrom<arguments::Named<Ipld>> for Send {
    type Error = ();

    fn try_from(named: arguments::Named<Ipld>) -> Result<Send, Self::Error> {
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

        Ok(Send {
            to: to.ok_or(())?,
            from: from.ok_or(())?,
            message: message.ok_or(())?,
        })
    }
}

impl TryFrom<arguments::Named<ipld::Promised>> for PromisedSend {
    type Error = ();

    fn try_from(args: arguments::Named<ipld::Promised>) -> Result<PromisedSend, Self::Error> {
        let mut to = None;
        let mut from = None;
        let mut message = None;

        for (key, prom) in args.0 {
            match key.as_str() {
                "to" => match Ipld::try_from(prom) {
                    Ok(Ipld::String(s)) => {
                        to = Some(promise::Any::Resolved(
                            url::Newtype::parse(s.as_str()).map_err(|_| ())?,
                        ));
                    }
                    Err(pending) => to = Some(pending.into()),
                    _ => return Err(()),
                },
                "from" => match Ipld::try_from(prom) {
                    Ok(Ipld::String(s)) => {
                        from = Some(promise::Any::Resolved(
                            url::Newtype::parse(s.as_str()).map_err(|_| ())?,
                        ));
                    }
                    Err(pending) => from = Some(pending.into()),
                    _ => return Err(()),
                },
                "message" => match Ipld::try_from(prom) {
                    Ok(Ipld::String(s)) => message = Some(promise::Any::Resolved(s)),
                    Err(pending) => to = Some(pending.into()),
                    _ => return Err(()),
                },
                _ => return Err(()),
            }
        }

        Ok(PromisedSend {
            to: to.ok_or(())?,
            from: from.ok_or(())?,
            message: message.ok_or(())?,
        })
    }
}

impl From<PromisedSend> for arguments::Named<Ipld> {
    fn from(p: PromisedSend) -> Self {
        arguments::Named::from_iter([
            ("to".into(), p.to.into()),
            ("from".into(), p.from.into()),
            ("message".into(), p.message.into()),
        ])
    }
}

const COMMAND: &'static str = "/msg/send";

impl Command for Send {
    const COMMAND: &'static str = COMMAND;
}

impl Command for PromisedSend {
    const COMMAND: &'static str = COMMAND;
}

impl From<Send> for PromisedSend {
    fn from(r: Send) -> Self {
        PromisedSend {
            to: promise::Any::Resolved(r.to),
            from: promise::Any::Resolved(r.from),
            message: promise::Any::Resolved(r.message),
        }
    }
}

impl TryFrom<PromisedSend> for Send {
    type Error = PromisedSend;

    fn try_from(p: PromisedSend) -> Result<Send, PromisedSend> {
        match p {
            PromisedSend {
                to: promise::Any::Resolved(to),
                from: promise::Any::Resolved(from),
                message: promise::Any::Resolved(message),
            } => Ok(Send { to, from, message }),
            _ => Err(p),
        }
    }
}

impl From<PromisedSend> for arguments::Named<ipld::Promised> {
    fn from(p: PromisedSend) -> Self {
        arguments::Named::from_iter([
            ("to".into(), p.to.to_promised_ipld()),
            ("from".into(), p.from.to_promised_ipld()),
            ("message".into(), p.message.to_promised_ipld()),
        ])
    }
}
