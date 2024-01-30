use crate::{
    ability::traits::{Command, Delegatable, Resolvable},
    promise::Deferrable,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use url::Url;

use super::any as msg;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Generic<To, From, Message> {
    pub to: To,
    pub from: From,
    pub message: Message,
}

pub type Resolved = Generic<Url, Url, String>;
pub type Builder = Generic<Option<Url>, Option<Url>, Option<String>>;
pub type Awaiting = Generic<Deferrable<Url>, Deferrable<Url>, Deferrable<String>>;

impl Delegatable for Resolved {
    type Builder = Builder;
}

impl Resolvable for Resolved {
    type Awaiting = Awaiting;
}

impl From<Awaiting> for Builder {
    fn from(awaiting: Awaiting) -> Self {
        Builder {
            to: awaiting.to.try_extract().ok(),
            from: awaiting.from.try_extract().ok(),
            message: awaiting.message.try_extract().ok(),
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
    type Parents = msg::Any;
    type ParentError = <msg::Any as CheckSame>::Error;

    // FIXME rename other to proof
    fn check_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        self.from.check_same(&other.from).map_err(|_| ())
    }
}

impl From<Resolved> for Builder {
    fn from(resolved: Resolved) -> Self {
        Generic {
            to: resolved.to.into(),
            from: resolved.from.into(),
            message: resolved.message.into(),
        }
    }
}

impl From<Resolved> for Awaiting {
    fn from(resolved: Resolved) -> Self {
        Generic {
            to: resolved.to.into(),
            from: resolved.from.into(),
            message: resolved.message.into(),
        }
    }
}

impl TryFrom<Awaiting> for Resolved {
    type Error = ();

    fn try_from(awaiting: Awaiting) -> Result<Self, ()> {
        Ok(Generic {
            to: awaiting.to.try_extract().map_err(|_| ())?,
            from: awaiting.from.try_extract().map_err(|_| ())?,
            message: awaiting.message.try_extract().map_err(|_| ())?,
        })
    }
}

impl TryFrom<Builder> for Resolved {
    type Error = ();

    fn try_from(builder: Builder) -> Result<Self, ()> {
        Ok(Generic {
            to: builder.to.ok_or(())?,
            from: builder.from.ok_or(())?,
            message: builder.message.ok_or(())?,
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
