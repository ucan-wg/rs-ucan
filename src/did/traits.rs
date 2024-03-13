use did_url::DID;
use std::fmt;

pub trait Did:
    PartialEq + ToString + TryFrom<DID> + Into<DID> + signature::Verifier<Self::Signature> + Ord
{
    type Signature: signature::SignatureEncoding + PartialEq + fmt::Debug;
    type Signer: signature::Signer<Self::Signature> + fmt::Debug;
}

pub trait Verifiable<DID: Did> {
    fn verifier<'a>(&'a self) -> &'a DID;
}
