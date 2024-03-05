use crate::{crypto::varsig, delegation::Delegation, did::Did};
use libipld_core::{cid::Cid, codec::Codec, link::Link};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct Proof<DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>> {
    pub prf: Vec<Link<Delegation<DID, V, Enc>>>,
}

impl<DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>> Serialize
    for Proof<DID, V, Enc>
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let chain = self
            .prf
            .iter()
            .map(|link| link.to_string())
            .collect::<Vec<_>>();

        chain.serialize(serializer)
    }
}

impl<'de, DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>> Deserialize<'de>
    for Proof<DID, V, Enc>
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let prf = Vec::<String>::deserialize(deserializer)?
            .into_iter()
            .map(|s| {
                let cid: Cid = s.try_into().map_err(serde::de::Error::custom)?;
                Ok(cid.into())
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Proof { prf })
    }
}
