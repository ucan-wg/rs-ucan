use crate::{crypto::varsig, delegation, did::Did, ipld};
use libipld_core::{codec::Codec, ipld::Ipld};

pub struct Pipe<DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>> {
    pub source: Cap<DID, V, Enc>,
    pub sink: Cap<DID, V, Enc>,
}

pub enum Cap<DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>> {
    Chain(delegation::Chain<DID, V, Enc>),
    Literal(Ipld),
}

pub struct PromisedPipe<DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>> {
    pub source: PromisedCap<DID, V, Enc>,
    pub sink: PromisedCap<DID, V, Enc>,
}

pub enum PromisedCap<DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>> {
    Chain(delegation::Chain<DID, V, Enc>),
    Promised(ipld::Promised),
}
