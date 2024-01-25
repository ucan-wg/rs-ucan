use crate::{
    ability::traits::{Ability, Builder},
    delegation,
    delegation::{Delegate, Delegation},
    receipt::Receipt,
    signature,
    time::Timestamp,
};
use did_url::DID;
use libipld_core::{ipld::Ipld, link::Link};
use std::collections::BTreeMap;

pub type Invocation<T, B, C> = signature::Envelope<Payload<T, B, C>>;

// FIXME figure out how to get rid of (AKA imply) that B param
#[derive(Debug, Clone, PartialEq)]
pub struct Payload<T, B, C>
where
    B: Builder<Concrete = T>,
{
    pub issuer: DID,
    pub subject: DID,
    pub audience: Option<DID>,

    pub ability: T, // TODO check name in spec

    pub proofs: Vec<Link<Delegation<B, C>>>,
    pub cause: Option<Link<Receipt<T, B, C>>>,
    pub metadata: BTreeMap<String, Ipld>, // FIXME serde value instead?
    pub nonce: Vec<u8>,                   // Better type?

    pub expiration: Timestamp,
    pub not_before: Option<Timestamp>,
}

// FIXME move that clone?
impl<T: Clone, B: Builder<Concrete = T> + From<T>, C> From<&Payload<T, B, C>>
    for delegation::Payload<B, C>
{
    fn from(invocation: &Payload<T, B, C>) -> Self {
        Self {
            issuer: invocation.issuer.clone(),
            subject: invocation.subject.clone(),
            audience: invocation
                .audience
                .clone()
                .unwrap_or(invocation.issuer.clone()),
            ability_builder: Delegate::Specific(invocation.ability.clone().into()),
            conditions: vec![],
            metadata: invocation.metadata.clone(),
            nonce: invocation.nonce.clone(),
            expiration: invocation.expiration.clone(),
            not_before: invocation.not_before.clone(),
        }
    }
}

impl<T, B: Builder<Concrete = T>, C> signature::Capsule for Payload<T, B, C> {
    const TAG: &'static str = "ucan/i/1.0.0-rc.1";
}
