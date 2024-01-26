use cid::Cid;
use libipld_core::ipld::Ipld;
use std::fmt::Debug;

/// A [`Promise`] is a way to defer the presence of a value to the result of some [`Invocation`].
#[derive(Debug, Clone, PartialEq)]
pub enum Promise {
    PromiseAny(Cid), // FIXME not sure about specifying the A here
    PromiseOk(Cid),
    PromiseErr(Cid),
}

impl TryFrom<Ipld> for Promise {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::Map(btree) = ipld {
            if btree.len() != 1 {
                return Err(());
            }

            if let Some(Ipld::Link(link)) = btree.get("await/ok") {
                return Ok(Self::PromiseOk(link.clone().into()));
            }

            if let Some(Ipld::Link(link)) = btree.get("await/err") {
                return Ok(Self::PromiseErr(link.clone().into()));
            }

            if let Some(Ipld::Link(link)) = btree.get("await/*") {
                return Ok(Self::PromiseAny(link.clone().into()));
            }
        }

        Err(())
    }
}
