use crate::{
    ability::{
        any::DelegateAny,
        traits::{Ability, Builder},
    },
    signature,
    time::Timestamp,
};
use did_url::DID;
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;

pub type Delegation<B, C> = signature::Envelope<Payload<B, C>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<B: Builder, C> {
    pub issuer: DID,
    pub subject: DID,
    pub audience: DID,

    pub capability_builder: Delegate<B>, // FIXME
    pub conditions: Vec<C>,              // Worth it over a Vec?

    pub metadata: BTreeMap<String, Ipld>, // FIXME serde value instead?
    pub nonce: Vec<u8>,                   // Better type?

    pub expiration: Timestamp,
    pub not_before: Option<Timestamp>,
}

impl<B: Builder, C> signature::Capsule for Payload<B, C> {
    const TAG: &'static str = "ucan/d/1.0.0-rc.1";
}

#[derive(Debug, Clone, PartialEq)]
pub enum Delegate<T> {
    Any,
    Specific(T),
}

impl<B: Builder + Clone, C: Clone> From<&Payload<B, C>> for Ipld
where
    Ipld: From<B> + From<C>,
{
    fn from(payload: &Payload<B, C>) -> Self {
        let mut map = BTreeMap::new();
        map.insert("iss".into(), payload.issuer.to_string().into());
        map.insert("sub".into(), payload.subject.to_string().into());
        map.insert("aud".into(), payload.audience.to_string().into());

        let can = match &payload.capability_builder {
            Delegate::Any => "ucan/*".into(),
            Delegate::Specific(builder) => builder.command().clone().into(),
        };

        map.insert("can".into(), can);

        map.insert(
            "args".into(),
            match &payload.capability_builder {
                Delegate::Any => Ipld::Map(BTreeMap::new()),
                Delegate::Specific(builder) => builder.clone().into(),
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
