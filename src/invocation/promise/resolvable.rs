use crate::{
    ability::{
        arguments,
        command::{ParseAbility, ToCommand},
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
    // type Promised: Into<Self::Builder> + Into<arguments::Named<ipld::Promised>>;
    type Promised: Into<arguments::Named<ipld::Promised>> + ToCommand;

    /// Attempt to resolve the [`Self::Promised`].
    fn try_resolve(promised: Self::Promised) -> Result<Self, CantResolve<Self>>
    where
        Self::Promised: Clone,
    {
        let ipld_promise: arguments::Named<ipld::Promised> = promised.clone().into();
        match arguments::Named::<Ipld>::try_from(ipld_promise) {
            Err(_) => Err(CantResolve {
                promised,
                reason: todo!(), // ParseAbility::ArgsErr::ExpectedMap,
            }),
            Ok(named) => {
                let builder = Self::Builder::try_parse(promised.to_command().as_str(), named)
                    .map_err(|reason| CantResolve {
                        promised: promised.clone(),
                        reason: todo!(),
                    })?;

                builder.try_into().map_err(|_reason| CantResolve {
                    promised,
                    reason: todo!(),
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

    fn try_to_builder(promised: Self::Promised) -> Result<Self::Builder, ()> {
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

        Self::Builder::try_parse(&cmd, named).map_err(|_| ())
    }
}

#[derive(Error)]
pub struct CantResolve<S: Resolvable> {
    pub promised: S::Promised,
    pub reason: <<S as Delegable>::Builder as ParseAbility>::ArgsErr,
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
