use super::{condition::Condition, delegate::Delegate};
use crate::{
    ability::traits::{Command, Delegatable, DynJs},
    capsule::Capsule,
    did::Did,
    nonce::Nonce,
    prove::TryProve,
    time::Timestamp,
};
use did_url::DID;
use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};
use web_time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub struct Payload<T: Delegatable, C: Condition> {
    pub issuer: Did,
    pub subject: Did,
    pub audience: Did,

    pub ability_builder: T::Builder,
    pub conditions: Vec<C>,

    pub metadata: BTreeMap<String, Ipld>,
    pub nonce: Nonce,

    pub expiration: Timestamp,
    pub not_before: Option<Timestamp>,
}

impl<T: Delegatable, C: Condition> Capsule for Payload<T, C>
where
    T::Builder: serde::Serialize + serde::de::DeserializeOwned,
{
    const TAG: &'static str = "ucan/d/1.0.0-rc.1";
}

// FIXME
impl<T: Delegatable, C: Condition + serde::Serialize + serde::de::DeserializeOwned> TryFrom<Ipld>
    for Payload<T, C>
where
    T::Builder: serde::Serialize + serde::de::DeserializeOwned,
{
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).unwrap() // FIXME
    }
}

// impl<C: Condition + Into<Ipld> + Clone> From<Payload<DynJs, C>> for Ipld {
//     fn from(payload: Payload<DynJs, C>) -> Self {
//         // FIXME I bet this clone can be removed by switcing to &DynJs
//         if let Ipld::Map(mut map) = payload.clone().into() {
//             map.insert("cmd".into(), payload.ability_builder.cmd.into());
//             map.insert("args".into(), payload.ability_builder.args.into());
//             map.into()
//         } else {
//             panic!("FIXME")
//         }
//     }
// }

impl<T: Delegatable + Command, C: Condition + Into<Ipld> + Clone> From<Payload<T, C>> for Ipld
where
    Ipld: From<T::Builder>,
{
    fn from(payload: Payload<T, C>) -> Self {
        let can = match &payload.ability_builder {
            Delegate::Any => "ucan/*".into(),
            Delegate::Specific(builder) => T::COMMAND.into(),
        };

        let mut map = BTreeMap::from_iter([
            ("iss".into(), payload.issuer.to_string().into()),
            ("sub".into(), payload.subject.to_string().into()),
            ("aud".into(), payload.audience.to_string().into()),
            (
                "args".into(),
                match &payload.ability_builder {
                    Delegate::Any => Ipld::Map(BTreeMap::new()),
                    Delegate::Specific(builder) => (*builder).clone().into(), // FIXME
                },
            ),
            ("cmd".into(), can),
            (
                "cond".into(),
                payload
                    .conditions
                    .iter()
                    .map(|condition| (*condition).clone().into())
                    .collect::<Vec<Ipld>>()
                    .into(),
            ),
            (
                "meta".into(),
                payload
                    .metadata
                    .clone()
                    .into_iter()
                    .map(|(key, value)| (key, value.into()))
                    .collect::<BTreeMap<String, Ipld>>()
                    .into(),
            ),
            ("nonce".into(), payload.nonce.clone().into()),
            ("exp".into(), payload.expiration.clone().into()),
        ]);

        if let Some(not_before) = &payload.not_before {
            map.insert("nbf".into(), not_before.clone().into());
        }

        map.into()
    }
}
