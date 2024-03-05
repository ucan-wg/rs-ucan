// use crate::{crypto::varsig, delegation::Delegation, did::Did};
// use libipld_core::{cid::Cid, codec::Codec, ipld::Ipld};
// use std::collections::BTreeMap;
//
// #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
// pub struct Batch<A, DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>> {
//     pub batch: Vec<Step<A, DID, V, Enc>>, // FIXME not quite right; would be nice to include meta etc
// }
//
// pub struct Step<A, DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>> {
//     pub subject: DID,
//     pub audience: Option<DID>,
//     pub ability: A, // FIXME promise version instead? Promised version shoudl be able to promise any field
//     pub cause: Option<Cid>,
//     pub metadata: BTreeMap<String, Ipld>,
//
//     pub cap: Vec<Delegation<DID, V, Enc>>,
// }
