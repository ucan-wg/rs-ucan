use super::enriched::Enriched;
use crate::invocation::promise::{Promise, PromiseAny, PromiseErr, PromiseOk};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};

/// A promise to recursively resolve to an [`Ipld`] value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Promised(pub Promise<Enriched<Promised>, Enriched<Promised>>);

impl From<Promise<Enriched<Promised>, Enriched<Promised>>> for Promised {
    fn from(promise: Promise<Enriched<Promised>, Enriched<Promised>>) -> Self {
        Promised(promise)
    }
}

impl From<PromiseAny<Enriched<Promised>, Enriched<Promised>>> for Promised {
    fn from(p_any: PromiseAny<Enriched<Promised>, Enriched<Promised>>) -> Self {
        Promised(p_any.into())
    }
}

impl From<PromiseOk<Enriched<Promised>>> for Promised {
    fn from(p_ok: PromiseOk<Enriched<Promised>>) -> Self {
        Promised(p_ok.into())
    }
}

impl From<PromiseErr<Enriched<Promised>>> for Promised {
    fn from(p_err: PromiseErr<Enriched<Promised>>) -> Self {
        Promised(p_err.into())
    }
}

// IPLD

impl From<Ipld> for Promised {
    fn from(ipld: Ipld) -> Self {
        Promised(Promise::Ok(PromiseOk::Fulfilled(ipld.into())))
    }
}

//  FIXME THIS is a great example of a try_resolve
//  FIXME is this recursive? Will this blow the stack?
impl TryFrom<Promised> for Ipld {
    type Error = Promised;

    fn try_from(p: Promised) -> Result<Ipld, Promised> {
        match p.0 {
            Promise::Ok(p_ok) => match p_ok {
                PromiseOk::Fulfilled(inner) => {
                    inner.try_into().map_err(|e| PromiseOk::Fulfilled(e).into())
                }

                PromiseOk::Pending(inner) => Err(PromiseOk::Pending(inner).into()),
            },
            Promise::Err(p_err) => match p_err {
                PromiseErr::Rejected(inner) => {
                    inner.try_into().map_err(|e| PromiseErr::Rejected(e).into())
                }

                PromiseErr::Pending(inner) => Err(PromiseErr::Pending(inner).into()),
            },
            Promise::Any(p_any) => match p_any {
                PromiseAny::Fulfilled(inner) => inner
                    .try_into()
                    .map_err(|e| Promise::Any(PromiseAny::Fulfilled(e)).into()),

                PromiseAny::Rejected(inner) => {
                    inner.try_into().map_err(|e| PromiseAny::Rejected(e).into())
                }

                PromiseAny::Pending(inner) => Err(PromiseAny::Pending(inner).into()),
            },
        }
    }
}

// FIXME surely the other version in this module can't be right if this also works?
// FIXME this is more iterative right?
// impl TryFrom<Promised> for Ipld {
//     // impl Resolvable for Ipld {
//     type Error = Self;
//     // type Promised = Promised;
//
//     fn try_from(promised: Promised) -> Result<Self, Self::Error> {
//         fn handle(enriched: super::Enriched<Promised>) -> Result<Ipld, ()> {
//             enriched
//                 .into_iter()
//                 .try_fold(vec![], |mut acc, next| {
//                     match next {
//                         Item::Inner(promised) => {
//                             let inner: Ipld = Resolvable::try_resolve(*promised).map_err(|_| ())?;
//
//                             acc.push(inner);
//                         }
//                         Item::Node(node) => {
//                             let _ = Ipld::try_from(*node).map_err(|_| ())?;
//                         }
//                     }
//                     Ok(acc)
//                 })
//                 .map(|vec| vec.first().expect("FIXME").clone())
//         }
//
//         match promised.0 {
//             Promise::Ok(promise_ok) => match promise_ok {
//                 PromiseOk::Fulfilled(enriched) => {
//                     handle(enriched).map_err(|_| PromiseOk::Fulfilled(enriched).into())
//                 }
//                 PromiseOk::Pending(_) => Err(promised),
//             },
//             Promise::Err(promise_err) => match promise_err {
//                 PromiseErr::Rejected(enriched) => {
//                     handle(enriched).map_err(|_| PromiseErr::Rejected(enriched).into())
//                 }
//                 PromiseErr::Pending(_) => Err(promised),
//             },
//             Promise::Any(promise_any) => match promise_any {
//                 PromiseAny::Fulfilled(enriched) => {
//                     handle(enriched).map_err(|_| PromiseAny::Fulfilled(enriched).into())
//                 }
//                 PromiseAny::Rejected(enriched) => {
//                     handle(enriched).map_err(|_| PromiseAny::Rejected(enriched).into())
//                 }
//                 PromiseAny::Pending(_) => Err(promised),
//             },
//         }
//     }
// }
