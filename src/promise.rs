use crate::{ability::traits::Buildable, invocation, invocation::Invocation, signature::Capsule};
use cid::Cid;
use libipld_core::{ipld::Ipld, link::Link};
use std::fmt::Debug;

// /// A [`Promise`] is a way to defer the presence of a value to the result of some [`Invocation`].
// #[derive(Debug, Clone, PartialEq)]
// pub enum Promise<T, A>
// where
//     A: Ability, // FIXME MUST be an Invocation
//     invocation::Payload<A>: Capsule,
// {
//     PromiseAny(Link<Invocation<A>>), // FIXME not sure about specifying the A here
//     PromiseOk(Link<Invocation<A>>),
//     PromiseErr(Link<Invocation<A>>),
// }

/// A [`Promise`] is a way to defer the presence of a value to the result of some [`Invocation`].
#[derive(Debug, Clone, PartialEq)]
pub enum Promise {
    PromiseAny(Cid), // FIXME not sure about specifying the A here
    PromiseOk(Cid),
    PromiseErr(Cid),
}

// impl<A: Ability> TryFrom<Ipld> for Promise<A>
// where
//     invocation::Payload<A>: Capsule,
// {
//     type Error = (); // FIXME
//
//     fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
//         if let Ipld::Map(btree) = ipld {
//             if let Some(Ipld::Link(link)) = btree.get("await/ok") {
//                 return Ok(Self::PromiseOk(link.clone().into()));
//             } else if let Some(Ipld::Link(link)) = btree.get("await/err") {
//                 return Ok(Self::PromiseErr(link.clone().into()));
//             } else if let Some(Ipld::Link(link)) = btree.get("await/*") {
//                 return Ok(Self::PromiseAny(link.clone().into()));
//             }
//         }
//
//         Err(())
//     }
// }
