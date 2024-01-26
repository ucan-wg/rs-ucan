use crate::{
    ability::traits::{Command, Delegatable, DynJs, JsHack},
    capsule::Capsule,
    delegation,
    delegation::{condition::Condition, Delegate},
    nonce::Nonce,
    time::Timestamp,
};
use did_url::DID;
use libipld_core::{cid::Cid, ipld::Ipld};
use std::{collections::BTreeMap, fmt::Debug};

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<B: Delegatable + Debug> {
    pub issuer: DID,
    pub subject: DID,
    pub audience: Option<DID>,

    pub ability: B,

    pub proofs: Vec<Cid>,
    pub cause: Option<Cid>,
    pub metadata: BTreeMap<String, Ipld>, // FIXME parameterize?
    pub nonce: Nonce,

    pub expiration: Timestamp,
    pub not_before: Option<Timestamp>,
}

// FIXME move that clone?
impl<B: Delegatable + Debug + Clone, C: Condition> From<&Payload<B>> for delegation::Payload<B, C> {
    fn from(invocation: &Payload<B>) -> Self {
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

impl<B: Delegatable + Debug> Capsule for Payload<B> {
    const TAG: &'static str = "ucan/i/1.0.0-rc.1";
}

impl<B: Delegatable + Command + Debug + Into<Ipld>> From<Payload<B>> for Ipld {
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

        map.insert("cmd".into(), B::COMMAND.into());
        map.insert("args".into(), payload.ability.into());

        map.insert(
            "proofs".into(),
            Ipld::List(
                payload
                    .proofs
                    .into_iter()
                    .map(|cid| cid.into())
                    .collect::<Vec<_>>(),
            ),
        );

        map.insert(
            "cause".into(),
            payload
                .cause
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

// FIXME TEMPORARY HACK to prove out proof of concept
impl From<Payload<DynJs>> for Ipld {
    fn from(payload: Payload<DynJs>) -> Self {
        let hack_payload = Payload {
            issuer: payload.issuer,
            subject: payload.subject,
            audience: payload.audience,
            ability: JsHack(payload.ability.clone()),
            proofs: payload.proofs,
            cause: payload.cause,
            metadata: payload.metadata,
            nonce: payload.nonce,
            expiration: payload.expiration,
            not_before: payload.not_before,
        };

        if let Ipld::Map(mut map) = hack_payload.into() {
            map.insert("cmd".into(), payload.ability.cmd.into());
            Ipld::Map(map)
        } else {
            // FIXME bleh this code
            unreachable!()
        }
    }
}
