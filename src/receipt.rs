use crate::{
    ability::traits::{Ability, Builder},
    delegation::Delegation,
    invocation::Invocation,
    signature,
    time::Timestamp,
};
use did_url::DID;
use libipld_core::{ipld::Ipld, link::Link};
use std::collections::BTreeMap;

pub type Receipt<T, B, C> = signature::Envelope<Payload<T, B, C>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<T, B: Builder<Concrete = T>, C> {
    pub issuer: DID,
    pub ran: Link<Invocation<T, B, C>>,
    pub out: Result<T, BTreeMap<String, Ipld>>,

    // pub proofs: Vec<Link<Delegation<B, C>>>, // FIXME these can only be executiojn proofs, right?
    pub metadata: BTreeMap<String, Ipld>,
    pub issued_at: Option<Timestamp>,
}

// #[derive(Debug, Clone, PartialEq)]
// pub enum UcanResult<T> {
//     UcanOk(T),
//     UcanErr(BTreeMap<String, Ipld>),
// }

impl<T, B: Builder<Concrete = T>, C> signature::Capsule for Payload<T, B, C> {
    const TAG: &'static str = "ucan/r/1.0.0-rc.1"; // FIXME extract out version
}
