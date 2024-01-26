use crate::{
    ability::traits::{Command, Delegatable, Runnable},
    capsule::Capsule,
    nonce::Nonce,
    signature,
    time::Timestamp,
};
use did_url::DID;
use libipld_core::{cid::Cid, ipld::Ipld};
use std::{collections::BTreeMap, fmt::Debug};

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<B: Runnable + Debug> {
    pub issuer: DID,

    pub ran: Cid,
    pub out: Result<B::Output, BTreeMap<String, Ipld>>,
    pub next: Vec<Cid>,

    pub proofs: Vec<Cid>,
    pub metadata: BTreeMap<String, Ipld>,

    pub nonce: Nonce,
    pub issued_at: Option<Timestamp>,
}

impl<B: Runnable + Debug> Capsule for Payload<B> {
    const TAG: &'static str = "ucan/r/1.0.0-rc.1"; // FIXME extract out version
}
