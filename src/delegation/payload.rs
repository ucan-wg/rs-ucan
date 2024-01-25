use super::{condition::Condition, delegate::Delegate};
use crate::{
    ability::traits::{Buildable, Command},
    signature,
    time::Timestamp,
};
use did_url::DID;
use libipld_core::ipld::Ipld;
use std::{collections::BTreeMap, fmt::Debug};

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<B: Buildable, C: Condition> {
    pub issuer: DID,
    pub subject: DID,
    pub audience: DID,

    pub ability_builder: Delegate<B::Builder>,
    pub conditions: Vec<C>,
    // pub fixme: Vec<Box<dyn Condition>>,
    pub metadata: BTreeMap<String, Ipld>,
    pub nonce: Vec<u8>,

    pub expiration: Timestamp,
    pub not_before: Option<Timestamp>,
}

impl<B: Buildable, C: Condition> signature::Capsule for Payload<B, C> {
    const TAG: &'static str = "ucan/d/1.0.0-rc.1";
}

impl<B: Buildable + Clone, C: Condition + Into<Ipld> + Clone> From<&Payload<B, C>> for Ipld
where
    Ipld: From<B::Builder>,
    B::Builder: Clone, // FIXME
{
    fn from(payload: &Payload<B, C>) -> Self {
        let mut map = BTreeMap::new();
        map.insert("iss".into(), payload.issuer.to_string().into());
        map.insert("sub".into(), payload.subject.to_string().into());
        map.insert("aud".into(), payload.audience.to_string().into());

        let can = match &payload.ability_builder {
            Delegate::Any => "ucan/*".into(),
            Delegate::Specific(builder) => builder.command().into(),
        };

        map.insert("can".into(), can);

        map.insert(
            "args".into(),
            match &payload.ability_builder {
                Delegate::Any => Ipld::Map(BTreeMap::new()),
                Delegate::Specific(builder) => (*builder).clone().into(), // FIXME
            },
        );
        map.insert(
            "cond".into(),
            payload
                .conditions
                .iter()
                .map(|condition| (*condition).clone().into())
                .collect::<Vec<Ipld>>()
                .into(),
        );
        map.insert(
            "meta".into(),
            payload
                .metadata
                .clone()
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect::<BTreeMap<String, Ipld>>()
                .into(),
        );
        map.insert("nonce".into(), payload.nonce.clone().into());
        map.insert("exp".into(), payload.expiration.clone().into());

        if let Some(not_before) = &payload.not_before {
            map.insert("nbf".into(), not_before.clone().into());
        }

        map.into()
    }
}

impl<B: Buildable + Clone, C: Condition + Into<Ipld> + Clone> From<Payload<B, C>> for Ipld
where
    Ipld: From<B::Builder>,
{
    fn from(payload: Payload<B, C>) -> Self {
        let mut map = BTreeMap::new();
        map.insert("iss".into(), payload.issuer.to_string().into());
        map.insert("sub".into(), payload.subject.to_string().into());
        map.insert("aud".into(), payload.audience.to_string().into());

        let can = match &payload.ability_builder {
            Delegate::Any => "ucan/*".into(),
            Delegate::Specific(builder) => builder.command().into(),
        };

        map.insert("can".into(), can);

        map.insert(
            "args".into(),
            match payload.ability_builder {
                Delegate::Any => Ipld::Map(BTreeMap::new()),
                Delegate::Specific(builder) => builder.into(),
            },
        );
        map.insert(
            "cond".into(),
            payload
                .conditions
                .into_iter()
                .map(|condition| condition.into())
                .collect::<Vec<Ipld>>()
                .into(),
        );
        map.insert(
            "meta".into(),
            payload
                .metadata
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect::<BTreeMap<String, Ipld>>()
                .into(),
        );
        map.insert("nonce".into(), payload.nonce.into());
        map.insert("exp".into(), payload.expiration.into());

        if let Some(not_before) = payload.not_before {
            map.insert("nbf".into(), not_before.into());
        }

        map.into()
    }
}
