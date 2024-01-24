use crate::{ability::traits::Ability, signature, time::Timestamp};
use did_url::DID;
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;

pub type Delegation<Ability, Cond> = signature::Envelope<Payload<Ability, Cond>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<A: Ability, Cond> {
    pub issuer: DID,
    pub subject: DID,
    pub audience: DID,

    pub capability_builder: A::Builder, // FIXME
    pub conditions: Box<[Cond]>,        // Worth it over a Vec?

    pub metadata: BTreeMap<Box<str>, Ipld>, // FIXME serde value instead?
    pub nonce: Box<[u8]>,                   // Better type?

    pub expiration: Timestamp,
    pub not_before: Option<Timestamp>,
}

impl<T: Ability, C> signature::Capsule for Payload<T, C> {
    const TAG: &'static str = "ucan/d/1.0.0-rc.1";
}
