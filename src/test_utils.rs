use libipld::{
    cid::multihash::{Code, MultihashDigest, MultihashGeneric},
    codec_impl::IpldCodec,
};
use proptest::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct SomeCodec(pub IpldCodec);

impl Arbitrary for SomeCodec {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            Just(IpldCodec::Raw),
            Just(IpldCodec::DagCbor),
            Just(IpldCodec::DagJson),
            Just(IpldCodec::DagPb),
        ]
        .prop_map(SomeCodec)
        .boxed()
    }
}

#[derive(Eq, Copy, Clone, Debug, PartialEq)]
pub struct SomeMultihash(pub Code);

impl Default for SomeMultihash {
    fn default() -> Self {
        SomeMultihash(Code::Sha2_256)
    }
}

impl SomeMultihash {
    pub fn new(multihash: Code) -> Self {
        SomeMultihash(multihash)
    }
}

impl From<Code> for SomeMultihash {
    fn from(multihash: Code) -> Self {
        SomeMultihash(multihash)
    }
}

impl From<SomeMultihash> for Code {
    fn from(wrapper: SomeMultihash) -> Self {
        wrapper.0
    }
}

impl From<SomeMultihash> for u64 {
    fn from(wrapper: SomeMultihash) -> Self {
        wrapper.0.into()
    }
}

impl TryFrom<u64> for SomeMultihash {
    type Error = <Code as TryFrom<u64>>::Error;

    fn try_from(code: u64) -> Result<Self, Self::Error> {
        let inner = code.try_into()?;
        Ok(SomeMultihash(inner))
    }
}

impl MultihashDigest<64> for SomeMultihash {
    fn digest(&self, input: &[u8]) -> MultihashGeneric<64> {
        self.0.digest(input)
    }

    fn wrap(&self, digest: &[u8]) -> Result<MultihashGeneric<64>, Self::Error> {
        self.0.wrap(digest)
    }
}

impl Arbitrary for SomeMultihash {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        // Only the 256-bit variants for now
        prop_oneof![
            Just(Code::Sha2_256),
            Just(Code::Sha3_256),
            Just(Code::Keccak256),
            Just(Code::Blake2s256),
            Just(Code::Blake2b256),
            Just(Code::Blake3_256),
        ]
        .prop_map(SomeMultihash)
        .boxed()
    }
}
