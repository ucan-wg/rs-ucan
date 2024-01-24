use crate::{
    ability::traits::Ability, delegation::Delegation, invocation::Invocation, signature,
    time::Timestamp,
};
use did_url::DID;
use libipld_core::{ipld::Ipld, link::Link};
use std::collections::BTreeMap;

pub type Receipt<T, C> = signature::Envelope<Payload<T, C>>;

pub struct Payload<T, Cond>
where
    T: Ability,
{
    pub issuer: DID,
    pub ran: Link<Invocation<T>>,
    pub out: UcanResult<T>, // FIXME?
    pub proofs: Vec<Link<Delegation<T, Cond>>>,
    pub metadata: BTreeMap<String, Ipld>,
    pub issued_at: Option<Timestamp>,
}

pub enum UcanResult<T> {
    UcanOk(T),
    UcanErr(BTreeMap<String, Ipld>),
}

impl<T: Ability, C> signature::Capsule for Payload<T, C> {
    const TAG: &'static str = "ucan/r/1.0.0-rc.1"; // FIXME extract out versioh
}
