use crate::{
    ability::{
        arguments,
        command::ToCommand,
        parse::{ParseAbility, ParseAbilityError, ParsePromised},
    },
    delegation::Delegable,
    invocation::promise::Pending,
    ipld,
};
use libipld_core::{cid::Cid, ipld::Ipld};
use std::{collections::BTreeSet, fmt};
use thiserror::Error;

// FIXME rename "Unresolved"
// FIXME better name

/// A trait for [`Delegable`]s that can be deferred (by promises).
///
/// FIXME exmaples
pub trait Resolvable: Delegable {
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

    fn into_promised(self) -> Self::Promised
    where
        <Self::Promised as ParsePromised>::PromisedArgsError: fmt::Debug,
    {
        // FIXME In no way efficient... override where possible, or just cut the impl
        let builder = Self::Builder::from(self);
        let cmd = &builder.to_command();
        let named_ipld: arguments::Named<Ipld> = builder.into();
        let promised_ipld: arguments::Named<ipld::Promised> = named_ipld.into();
        <Self as Resolvable>::Promised::try_parse_promised(cmd, promised_ipld)
            .expect("promise to always be possible from a ready ability")
    }

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
                let builder = Self::Builder::try_parse(promised.to_command().as_str(), named)
                    .map_err(|_reason| CantResolve {
                        promised: promised.clone(),
                        reason: ResolveError::ConversionError,
                    })?;

                builder.try_into().map_err(|_reason| CantResolve {
                    promised,
                    reason: ResolveError::ConversionError,
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

    fn try_to_builder(
        promised: Self::Promised,
    ) -> Result<
        Self::Builder,
        ParseAbilityError<<<Self as Delegable>::Builder as ParseAbility>::ArgsErr>,
    > {
        let cmd = promised.to_command();
        let ipld_promise: arguments::Named<ipld::Promised> = promised.into();

        let named: arguments::Named<Ipld> =
            ipld_promise
                .into_iter()
                .fold(arguments::Named::new(), |mut acc, (k, v)| {
                    match v.try_into() {
                        Err(_) => (),
                        Ok(ipld) => {
                            acc.insert(k, ipld); // i.e. forget any promises
                        }
                    }

                    acc
                });

        Self::Builder::try_parse(&cmd, named)
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
    <<S as Delegable>::Builder as ParseAbility>::ArgsErr: fmt::Debug,
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

    #[error("The resolved promise was unable to reify a Builder")]
    ConversionError,
}
