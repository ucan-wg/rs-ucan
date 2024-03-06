//! Utilities for ability arguments

mod named;

pub use named::*;

use crate::{invocation::promise::Resolves, ipld};
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;

// FIXME move under invoc::promise?
// pub type Promised = Resolves<Named<ipld::Promised>>;
//
// impl Promised {
//     pub fn try_resolve_option(self) -> Option<Named<Ipld>> {
//         match self.try_resolve() {
//             Err(_) => None,
//             Ok(named_promises) => named_promises
//                 .iter()
//                 .try_fold(BTreeMap::new(), |mut map, (k, v)| {
//                     map.insert(k.clone(), Ipld::try_from(v.clone()).ok()?);
//                     Some(map)
//                 })
//                 .map(Named),
//         }
//     }
// }
