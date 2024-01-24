use crate::{ability::traits::Ability, delegation, time::Timestamp};
use did_url::DID;
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<Ability> {
    pub issuer: DID,
    pub subject: DID,
    pub audience: Option<DID>,

    pub ability: Ability, // FIXME check name in spec

    // pub proofs: Vec<Link<Delegation<Ability>>>,
    // pub cause: Option<Link<Receipt<_>>>, // FIXME?
    pub metadata: BTreeMap<Box<str>, Ipld>, // FIXME serde value instead?
    pub nonce: Box<[u8]>,                   // Better type?

    pub expiration: Timestamp,
    pub not_before: Option<Timestamp>,
}

// FIXME move that clone?
impl<A: Ability + Clone, Cond> From<Payload<A>> for delegation::Payload<A, Cond> {
    fn from(invocation: Payload<A>) -> Self {
        Self {
            issuer: invocation.issuer.clone(),
            subject: invocation.subject.clone(),
            audience: invocation
                .audience
                .clone()
                .unwrap_or(invocation.issuer.clone()),
            capability_builder: <A as Into<A::Builder>>::into(invocation.ability.clone()),
            conditions: Box::new([]),
            metadata: invocation.metadata.clone(),
            nonce: invocation.nonce.clone(),
            expiration: invocation.expiration.clone(),
            not_before: invocation.not_before.clone(),
        }
    }
}
