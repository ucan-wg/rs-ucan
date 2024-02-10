use super::enriched::Enriched;
use crate::invocation::promise::{Promise, PromiseAny, PromiseErr, PromiseOk};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};

/// A promise to recursively resolve to an [`Ipld`] value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Promised(pub Promise<Enriched<Promised>, Enriched<Promised>>);

impl Promised {
    // FIXME note that this is different from the failable version which is more like
    // a try_reoslve... which has a note at the bottom on this module
    pub fn serialize_as_ipld(&self) -> Ipld {
        ipld_serde::to_ipld(self).unwrap() // FIXME at worst we can do this by hand
    }
}

// Promise variants into Promised

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
