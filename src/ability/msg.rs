//! Message abilities

mod any;
mod receive;

pub mod send;

pub use any::Any;
pub use receive::Receive;

use crate::{
    ability::arguments,
    delegation::Delegable,
    invocation::Resolvable,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::ipld::Ipld;

// FIXME rename invokable?
#[derive(Debug, Clone, PartialEq)]
pub enum Ready {
    Send(send::Ready),
    Receive(receive::Receive),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Builder {
    Send(send::Builder),
    Receive(receive::Receive),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Promised {
    Send(send::Promised),
    Receive(receive::Promised), // FIXME
}

impl Delegable for Ready {
    type Builder = Builder;
}

impl TryFrom<Builder> for Ready {
    type Error = ();

    fn try_from(builder: Builder) -> Result<Self, Self::Error> {
        match builder {
            Builder::Send(send) => send.try_into().map(Ready::Send).map_err(|_| ()),
            Builder::Receive(receive) => Ok(Ready::Receive(receive)),
        }
    }
}

impl From<Ready> for Builder {
    fn from(ready: Ready) -> Self {
        match ready {
            Ready::Send(send) => Builder::Send(send.into()),
            Ready::Receive(receive) => Builder::Receive(receive.into()),
        }
    }
}

impl From<Promised> for arguments::Named<Ipld> {
    fn from(promised: Promised) -> Self {
        match promised {
            Promised::Send(send) => send.into(),
            Promised::Receive(receive) => receive.into(),
        }
    }
}

impl Resolvable for Ready {
    type Promised = Promised;

    fn try_resolve(promised: Promised) -> Result<Self, Self::Promised> {
        match promised {
            Promised::Send(send) => Resolvable::try_resolve(send)
                .map(Ready::Send)
                .map_err(Promised::Send),
            Promised::Receive(receive) => Resolvable::try_resolve(receive)
                .map(Ready::Receive)
                .map_err(Promised::Receive),
        }
    }
}

impl From<Promised> for Builder {
    fn from(promised: Promised) -> Self {
        match promised {
            Promised::Send(send) => Builder::Send(send.into()),
            Promised::Receive(receive) => Builder::Receive(receive.into()),
        }
    }
}

impl CheckSame for Builder {
    type Error = (); // FIXME

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        match (self, proof) {
            (Builder::Send(this), Builder::Send(that)) => this.check_same(that),
            (Builder::Receive(this), Builder::Receive(that)) => this.check_same(that),
            _ => Err(()),
        }
    }
}

impl CheckParents for Builder {
    type Parents = Any;
    type ParentError = ();

    fn check_parent(&self, proof: &Self::Parents) -> Result<(), Self::ParentError> {
        match (self, proof) {
            (Builder::Send(this), Any) => this.check_parent(&Any),
            (Builder::Receive(this), Any) => this.check_parent(&Any),
        }
    }
}

impl Checkable for Builder {
    type Hierarchy = Parentful<Builder>;
}
