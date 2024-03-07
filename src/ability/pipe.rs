use crate::{crypto::varsig, delegation, did::Did, ipld};
use libipld_core::{codec::Codec, ipld::Ipld};

pub struct Pipe<DID: Did, V: varsig::Header<C>, C: Codec + TryFrom<u32> + Into<u32>> {
    pub source: Cap<DID, V, C>,
    pub sink: Cap<DID, V, C>,
}

pub enum Cap<DID: Did, V: varsig::Header<C>, C: Codec + TryFrom<u32> + Into<u32>> {
    Proof(delegation::Proof<DID, V, C>),
    Literal(Ipld),
}

pub struct PromisedPipe<DID: Did, V: varsig::Header<C>, C: Codec + TryFrom<u32> + Into<u32>> {
    pub source: PromisedCap<DID, V, C>,
    pub sink: PromisedCap<DID, V, C>,
}

pub enum PromisedCap<DID: Did, V: varsig::Header<C>, C: Codec + TryFrom<u32> + Into<u32>> {
    Proof(delegation::Proof<DID, V, C>),
    Promised(ipld::Promised),
}
