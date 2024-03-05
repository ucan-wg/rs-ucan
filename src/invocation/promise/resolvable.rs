use crate::{
    ability::{
        arguments,
        command::ToCommand,
        parse::{ParseAbility, ParsePromised},
    },
    invocation::promise::Pending,
    ipld,
};
use libipld_core::{cid::Cid, ipld::Ipld};
use std::{collections::BTreeSet, fmt};
use thiserror::Error;

/// A trait for [`Delegable`]s that can be deferred (by promises).
///
/// FIXME exmaples
pub trait Resolvable: Sized + ParseAbility + ToCommand {
    /// The promise type that resolves to `Self`.
    ///
    /// Note that this may be a more complex type than the promise selector
    /// variants. One example is [letting any leaf][PromiseIpld] of an [`Ipld`] graph
    /// be a promise.
    ///
    /// [PromiseIpld]: crate::ipld::Promised
    type Promised: ToCommand
        + ParsePromised // TryFrom<arguments::Named<ipld::Promised>>
        + Into<arguments::Named<ipld::Promised>>;

    /// Attempt to resolve the [`Self::Promised`].
    fn try_resolve(promised: Self::Promised) -> Result<Self, CantResolve<Self>>
    where
        Self::Promised: Clone,
    {
        let ipld_promise: arguments::Named<ipld::Promised> = promised.clone().into();
        match arguments::Named::<Ipld>::try_from(ipld_promise) {
            Err(pending) => Err(CantResolve {
                promised,
                reason: ResolveError::StillWaiting(pending),
            }),
            Ok(named) => {
                ParseAbility::try_parse(&promised.to_command(), named).map_err(|_reason| {
                    CantResolve {
                        promised,
                        reason: ResolveError::ConversionError,
                    }
                })
            }
        }
    }

    fn get_all_pending(promised: Self::Promised) -> BTreeSet<Cid> {
        let promise_map: arguments::Named<ipld::Promised> = promised.into();

        promise_map
            .values()
            .fold(BTreeSet::new(), |mut set, promised| {
                if let ipld::Promised::Link(cid) = promised {
                    set.insert(*cid);
                }

                set
            })
    }
}

#[derive(Error, Clone)]
pub struct CantResolve<S: Resolvable> {
    pub promised: S::Promised,
    pub reason: ResolveError,
}

impl<S: Resolvable> fmt::Debug for CantResolve<S>
where
    S::Promised: fmt::Debug,
    Pending: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CantResolve")
            .field("promised", &self.promised)
            .field("reason", &self.reason)
            .finish()
    }
}

#[derive(Error, PartialEq, Eq, Clone, Debug)]
pub enum ResolveError {
    #[error("The promise is still has arguments waiting to be resolved")]
    StillWaiting(Pending),

    #[error("The resolved promise was unable to reify an ability from IPLD")]
    ConversionError,
}
