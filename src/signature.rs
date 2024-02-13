//! Signatures and cryptographic envelopes.

use crate::{capsule::Capsule, did::Did};
use libipld_core::{
    cid::{Cid, CidGeneric},
    codec::{Codec, Encode},
    ipld::Ipld,
    multihash::{Code, MultihashGeneric},
};
use serde::{Deserialize, Serialize};
use signature::SignatureEncoding;
use std::collections::BTreeMap;

// FIXME #[cfg(feature = "dag-cbor")]
use libipld_cbor::DagCborCodec;
use signature::Signer;

/// A container associating a `payload` with its signature over it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope<T: Capsule, S: SignatureEncoding> {
    /// The signture of the `payload`.
    pub signature: Signature<S>,

    /// The payload that's being signed over.
    pub payload: T,
}

impl<T: Capsule + Clone + Into<Ipld>, S: SignatureEncoding> Envelope<T, S> {
    pub fn try_sign<DID: Did>(
        signer: &DID::Signer,
        payload: T,
    ) -> Result<Envelope<T, <DID as Did>::Signature>, ()> {
        Self::try_sign_generic::<DagCborCodec, Code, DID>(
            signer,
            DagCborCodec,
            Code::Sha2_256,
            payload,
        )
    }

    pub fn try_sign_generic<C: Codec, H: Into<u64>, DID: Did>(
        signer: &DID::Signer,
        codec: C,
        hasher: H,
        payload: T,
    ) -> Result<Envelope<T, DID::Signature>, ()>
    // FIXME err = ()
    where
        Ipld: Encode<C>,
    {
        let ipld: Ipld = BTreeMap::from_iter([(T::TAG.into(), payload.clone().into())]).into();

        let mut buffer = vec![];
        ipld.encode(codec, &mut buffer)
            .expect("FIXME not dag-cbor? DagCborCodec to encode any arbitrary `Ipld`");

        let cid: Cid = CidGeneric::new_v1(
            codec.into(),
            MultihashGeneric::wrap(hasher.into(), buffer.as_slice())
                .map_err(|_| ()) // FIXME
                .expect("FIXME expect signing to work..."),
        );

        let sig = signer.try_sign(&cid.to_bytes()).map_err(|_| ())?;

        Ok(Envelope {
            signature: Signature::One(sig),
            payload,
        })
    }

    pub fn validate_signature(&self) -> Result<(), ()> {
        // FIXME
        todo!()
    }
}

// FIXME consider kicking Batch down the road for spec reasons?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Signature<S> {
    One(S),
    Batch {
        signature: S,
        merkle_proof: Vec<Vec<u8>>,
    },
}

impl<S: Into<Ipld>> From<Signature<S>> for Ipld {
    fn from(signature: Signature<S>) -> Self {
        match signature {
            Signature::One(sig) => sig.into(),
            Signature::Batch {
                signature,
                merkle_proof,
            } => Ipld::List(merkle_proof.into_iter().map(|p| p.into()).collect()),
        }
    }
}

impl<T: Capsule + Into<Ipld>, S: SignatureEncoding + Into<Ipld>> From<Envelope<T, S>> for Ipld {
    fn from(Envelope { signature, payload }: Envelope<T, S>) -> Self {
        let ipld: Ipld = BTreeMap::from_iter([(T::TAG.into(), payload.into())]).into();

        let codec = DagCborCodec; // FIXME get this from the payload
        let hasher = Code::Sha2_256; // FIXME get this from the payload

        let mut buffer = vec![];
        ipld.encode(codec, &mut buffer)
            .expect("FIXME not dag-cbor? DagCborCodec to encode any arbitrary `Ipld`");

        let cid = CidGeneric::new_v1(
            codec.into(),
            MultihashGeneric::wrap(hasher.into(), buffer.as_slice())
                .map_err(|_| ()) // FIXME
                .expect("FIXME expect signing to work..."),
        );

        BTreeMap::from_iter([("sig".into(), signature.into()), ("pld".into(), cid.into())]).into()
    }
}
