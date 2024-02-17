//! Signatures and cryptographic envelopes.

use crate::{capsule::Capsule, did::Did};
// use anyhow;
use libipld_core::{
    cid::{Cid, CidGeneric},
    codec::{Codec, Encode},
    ipld::Ipld,
    multihash::{Code, MultihashGeneric},
};
use std::collections::BTreeMap;

// FIXME #[cfg(feature = "dag-cbor")]
use libipld_cbor::DagCborCodec;
use signature::{SignatureEncoding, Signer};

pub trait Verifiable<DID: Did> {
    fn verifier<'a>(&'a self) -> &'a DID;
}

impl<T: Verifiable<DID> + Capsule, DID: Did> Verifiable<DID> for Envelope<T, DID> {
    fn verifier(&self) -> &DID {
        &self.payload.verifier()
    }
}

/// A container associating a `payload` with its signature over it.
#[derive(Debug, Clone, PartialEq)] // , Serialize, Deserialize)]
pub struct Envelope<T: Verifiable<DID> + Capsule, DID: Did> {
    /// The signture of the `payload`.
    pub signature: Signature<DID::Signature>,

    /// The payload that's being signed over.
    pub payload: T,
}

impl<T: Capsule + Verifiable<DID> + Into<Ipld> + Clone, DID: Did> Envelope<T, DID> {
    pub fn try_sign(signer: &DID::Signer, payload: T) -> Result<Envelope<T, DID>, ()> {
        Self::try_sign_generic::<DagCborCodec, Code>(signer, DagCborCodec, payload)
    }

    pub fn try_sign_generic<C: Codec, H: Into<u64>>(
        signer: &DID::Signer,
        codec: C,
        payload: T,
    ) -> Result<Envelope<T, DID>, ()>
    // FIXME err = ()
    where
        Ipld: Encode<C>,
    {
        let ipld: Ipld = BTreeMap::from_iter([(T::TAG.into(), payload.clone().into())]).into();

        let mut buffer = vec![];
        ipld.encode(codec, &mut buffer)
            .expect("FIXME not dag-cbor? DagCborCodec to encode any arbitrary `Ipld`");

        let sig = signer.try_sign(&buffer).map_err(|_| ())?;

        Ok(Envelope {
            signature: Signature::Solo(sig),
            payload,
        })
    }

    pub fn validate_signature(&self) -> Result<(), ()> {
        // FIXME need varsig
        let codec = DagCborCodec;
        let hasher = Code::Sha2_256;

        let mut buffer = vec![];
        let ipld: Ipld = BTreeMap::from_iter([(T::TAG.into(), self.payload.clone().into())]).into();
        ipld.encode(codec, &mut buffer)
            .expect("FIXME not dag-cbor? DagCborCodec to encode any arbitrary `Ipld`");

        let cid: Cid = CidGeneric::new_v1(
            codec.into(),
            MultihashGeneric::wrap(hasher.into(), buffer.as_slice())
                .map_err(|_| ()) // FIXME
                .expect("FIXME expect signing to work..."),
        );

        match &self.signature {
            Signature::Solo(sig) => self
                .verifier()
                .verify(&cid.to_bytes(), &sig)
                .map_err(|_| ()),
        }
    }
}

// FIXME consider kicking Batch down the road for spec reasons?
#[derive(Debug, Clone, PartialEq)] // , Serialize, Deserialize)]
                                   // #[serde(untagged)]
pub enum Signature<S> {
    Solo(S),
    // Batch {
    //     signature: S,
    //     root: Vec<u8>,
    //     merkle_proof: Vec<MerkleStep>,
    // },
}

//#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
//pub enum MerkleStep {
//    Node(Vec<u8>, Vec<u8>, Direction),
//    Turn(Direction),
//    End,
//}
//
//#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
//pub enum Direction {
//    Left = 0,
//    Right = 1,
//}

// impl<C: Codec, T: Verifiable<DID> + Capsule + Into<Ipld>, DID: Did> Encode<C> for Envelope<T, DID>
// where
//     Ipld: Encode<C>,
//     Envelope<T, DID>: Clone, // FIXME?
// {
//     fn encode<W: std::io::Write>(&self, codec: C, writer: &mut W) -> Result<(), anyhow::Error> {
//         Ipld::from((*self).clone()).encode(codec, writer)
//     }
// }

impl<S: SignatureEncoding> From<Signature<S>> for Ipld {
    fn from(signature: Signature<S>) -> Self {
        match signature {
            Signature::Solo(sig) => sig.to_vec().into(),
            // Signature::Batch {
            //     signature,
            //     merkle_proof,
            // } => Ipld::List(merkle_proof.into_iter().map(|p| p.into()).collect()),
        }
    }
}

impl<T: Verifiable<DID> + Capsule + Into<Ipld>, DID: Did> From<Envelope<T, DID>> for Ipld {
    fn from(Envelope { signature, payload }: Envelope<T, DID>) -> Self {
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
