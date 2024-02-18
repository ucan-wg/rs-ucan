//! The ability to send messages

use crate::{
    ability::{arguments, command::Command},
    delegation::Delegable,
    invocation::promise,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
    url as url_newtype,
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::BTreeMap;
use url::Url;

/// Helper for creating instances of `msg/send` with the correct shape.
///
/// This is not generally used directly, unless you want to abstract
/// over all of the `msg/send` variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Generic<To, From, Message> {
    /// The recipient of the message
    pub to: To,

    /// FIXME Builder needs to omit option fields from Serde

    /// The sender address of the message
    ///
    /// This *may* be a URL (such as an email address).
    /// If provided, the `subject` must have the right to send from this address.
    pub from: From,

    /// The main body of the message
    pub message: Message,
}

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
pub type Ready = Generic<Url, Url, String>;

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
pub type Builder = Generic<Option<Url>, Option<Url>, Option<String>>;

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
pub type Promised =
    Generic<promise::Resolves<Url>, promise::Resolves<Url>, promise::Resolves<String>>;

impl Delegable for Ready {
    type Builder = Builder;
}

impl promise::Resolvable for Ready {
    type Promised = Promised;

    fn try_resolve(p: Promised) -> Result<Self, Promised> {
        match promise::Resolves::try_resolve_3(p.to, p.from, p.message) {
            Ok((to, from, message)) => Ok(Ready { to, from, message }),
            Err((to, from, message)) => Err(Promised { to, from, message }),
        }
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
            ("to".into(), p.to.map(url_newtype::Newtype).into()),
            ("from".into(), p.from.map(String::from).into()),
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

impl<T, F, M> Command for Generic<T, F, M> {
    const COMMAND: &'static str = "msg/send";
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

impl From<Ready> for Builder {
    fn from(resolved: Ready) -> Self {
        Generic {
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

impl<T, F, M> From<Generic<T, F, M>> for Ipld
where
    Ipld: From<T> + From<F> + From<M>,
{
    fn from(send: Generic<T, F, M>) -> Self {
        send.into()
    }
}

impl<T: DeserializeOwned, F: DeserializeOwned, M: DeserializeOwned> TryFrom<Ipld>
    for Generic<T, F, M>
{
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}
