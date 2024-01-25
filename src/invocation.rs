use crate::{
    ability::traits::{Buildable, Command},
    delegation,
    delegation::{condition::Condition, Delegate, Delegation},
    receipt::Receipt,
    signature,
    time::Timestamp,
};
use did_url::DID;
use libipld_core::{cid::Cid, ipld::Ipld, link::Link};
use std::{collections::BTreeMap, fmt::Debug};

pub type Invocation<B> = signature::Envelope<Payload<B>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<B: Buildable + Debug> {
    pub issuer: DID,
    pub subject: DID,
    pub audience: Option<DID>,

    pub ability: B,

    // pub proofs: Vec<Link<Delegation<B, C>>>, // FIXME just use Cid?
    pub proofs: Vec<Cid>, // FIXME just use Cid?
    pub cause: Option<Cid>,
    pub metadata: BTreeMap<String, Ipld>, // FIXME serde value instead?
    pub nonce: Vec<u8>,                   // Better type?

    pub expiration: Timestamp,
    pub not_before: Option<Timestamp>,
}

// FIXME move that clone?
impl<B: Buildable + Debug + Clone, C: Condition> From<&Payload<B>> for delegation::Payload<B, C> {
    fn from(invocation: &Payload<B>) -> Self {
        Self {
            issuer: invocation.issuer.clone(),
            subject: invocation.subject.clone(),
            audience: invocation
                .audience
                .clone()
                .unwrap_or(invocation.issuer.clone()),
            ability_builder: Delegate::Specific(invocation.ability.clone().to_builder()),
            conditions: vec![],
            // fixme: vec![],
            metadata: invocation.metadata.clone(),
            nonce: invocation.nonce.clone(),
            expiration: invocation.expiration.clone(),
            not_before: invocation.not_before.clone(),
        }
    }
}

impl<B: Buildable + Debug> signature::Capsule for Payload<B> {
    const TAG: &'static str = "ucan/i/1.0.0-rc.1";
}

impl<B: Buildable + Debug + Into<Ipld>> From<Payload<B>> for Ipld {
    fn from(payload: Payload<B>) -> Self {
        let mut map = BTreeMap::new();
        map.insert("iss".into(), payload.issuer.to_string().into());
        map.insert("sub".into(), payload.subject.to_string().into());
        map.insert(
            "aud".into(),
            payload
                .audience
                .map(|audience| audience.to_string())
                .unwrap_or(payload.issuer.to_string())
                .into(),
        );

        map.insert("can".into(), payload.ability.command().into());
        map.insert("args".into(), payload.ability.into());

        map.insert(
            "proofs".into(),
            Ipld::List(
                payload
                    .proofs
                    .into_iter()
                    .map(|cid| cid.into())
                    // .map(|link| link.cid().into())
                    .collect::<Vec<_>>(),
            ),
        );

        map.insert(
            "cause".into(),
            payload
                .cause
                // .map(|link| link.cid().into())
                .map(|cid| cid.into())
                .unwrap_or(Ipld::Null),
        );

        map.insert(
            "metadata".into(),
            Ipld::Map(
                payload
                    .metadata
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<BTreeMap<String, Ipld>>(),
            ),
        );

        map.insert("nonce".into(), payload.nonce.into());

        map.insert("exp".into(), payload.expiration.into());

        if let Some(not_before) = payload.not_before {
            map.insert("nbf".into(), not_before.into());
        }

        map.into()
    }
}
