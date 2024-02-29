use crate::{crypto::varsig, delegation, delegation::condition::Condition, did::Did, ipld};
use libipld_core::{codec::Codec, ipld::Ipld};

pub struct Pipe<
    C: Condition,
    DID: Did,
    V: varsig::Header<Enc>,
    Enc: Codec + TryFrom<u32> + Into<u32>,
> {
    pub source: Cap<C, DID, V, Enc>,
    pub sink: Cap<C, DID, V, Enc>,
}

pub enum Cap<C: Condition, DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>>
{
    Chain(delegation::Chain<C, DID, V, Enc>),
    Literal(Ipld),
}

pub struct PromisedPipe<
    C: Condition,
    DID: Did,
    V: varsig::Header<Enc>,
    Enc: Codec + TryFrom<u32> + Into<u32>,
> {
    pub source: PromisedCap<C, DID, V, Enc>,
    pub sink: PromisedCap<C, DID, V, Enc>,
}

pub enum PromisedCap<
    C: Condition,
    DID: Did,
    V: varsig::Header<Enc>,
    Enc: Codec + TryFrom<u32> + Into<u32>,
> {
    Chain(delegation::Chain<C, DID, V, Enc>),
    Promised(ipld::Promised),
}
