use crate::capsule::Capsule;
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Envelope<T: Capsule> {
    pub sig: Signature,
    pub payload: T,
}

// FIXME consider kicking Batch down the road for spec reasons?
#[derive(Debug, Clone, PartialEq)]
pub enum Signature {
    One(Vec<u8>),
    Batch {
        sig: Vec<u8>,
        merkle_proof: Vec<Vec<u8>>,
    },
}

impl From<&Signature> for Ipld {
    fn from(sig: &Signature) -> Self {
        match sig {
            Signature::One(sig) => sig.clone().into(),
            Signature::Batch { sig, merkle_proof } => {
                let mut map = BTreeMap::new();
                let proof: Vec<Ipld> = merkle_proof.into_iter().map(|p| p.clone().into()).collect();
                map.insert("sig".into(), sig.clone().into());
                map.insert("prf".into(), proof.into());
                map.into()
            }
        }
    }
}

impl From<Signature> for Ipld {
    fn from(sig: Signature) -> Self {
        match sig {
            Signature::One(sig) => sig.into(),
            Signature::Batch { sig, merkle_proof } => {
                let mut map = BTreeMap::new();
                let proof: Vec<Ipld> = merkle_proof.into_iter().map(|p| p.into()).collect();
                map.insert("sig".into(), sig.into());
                map.insert("prf".into(), proof.into());
                map.into()
            }
        }
    }
}

// FIXME Store or BTreeMap? Also eliminate that Clone constraint
impl<T: Capsule + Into<Ipld> + Clone> From<&Envelope<T>> for Ipld {
    fn from(Envelope { sig, payload }: &Envelope<T>) -> Self {
        let mut inner = BTreeMap::new();
        inner.insert(T::TAG.into(), payload.clone().into()); // FIXME should be a link

        let mut map = BTreeMap::new();
        map.insert("sig".into(), sig.into());
        map.insert("pld".into(), Ipld::Map(inner));

        Ipld::Map(map)
    }
}

impl<T: Capsule + Into<Ipld> + Clone> From<Envelope<T>> for Ipld {
    fn from(Envelope { sig, payload }: Envelope<T>) -> Self {
        let mut inner = BTreeMap::new();
        inner.insert(T::TAG.into(), payload.clone().into()); // FIXME should be a link

        let mut map = BTreeMap::new();
        map.insert("sig".into(), sig.into());
        map.insert("pld".into(), Ipld::Map(inner));

        Ipld::Map(map)
    }
}
