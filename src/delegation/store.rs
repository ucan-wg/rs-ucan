use super::{condition::Condition, delegatable::Delegatable, Delegation};
use crate::did::Did;
use libipld_core::cid::Cid;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use web_time::SystemTime;

// NOTE can already look up by CID in other traits
pub trait IndexedStore<T: Delegatable, C: Condition> {
    type Error;

    fn get_by(query: Query) -> Result<HashMap<Cid, Delegation<T, C>>, Self::Error>;

    fn previously_checked(cid: Cid) -> Result<bool, Self::Error>;

    // NOTE you can override this with something much more efficient in e.g. SQL
    // FIXME "should" be checked and indexed on the way in
    fn chains_for(
        &self,
        subject: Did,
        command: String,
        audience: Did,
    ) -> Result<Vec<Vec<(Cid, Delegation<T, C>)>>, Self::Error>;
    //  if let Ok(possible) = self.get_by(Query {
    //      audience: Some(audience),
    //      command: Some(command),
    //      after_not_before: Some(SystemTime::now()),
    //      expires_before: Some(SystemTime::now()),
    //      ..Default::default()
    //  }) {
    //      let acc = Ok(vec![]);
    //      let iss = possible.iter().next().unwrap().1.issuer;

    //      // FIXME actually more complex than this:
    //      // sicne the chain also has to be valid
    //      // ...we shoud probably index on the way in
    //      while acc.is_ok() {
    //          if let Ok(latest) = get_one(Query {
    //              subject: Some(subject),
    //              command: Some(command),
    //              audience: Some(latest_iss),
    //              after_not_before: Some(SystemTime::now()),
    //              expires_before: Some(SystemTime::now()),
    //              ..Default::default()
    //          }) {
    //              acc.push(latest);

    //              if delegation.
    //              latest_iss = delegation.issuer;
    //          } else {
    //              acc = Err(()); // FIXME
    //          }
    //      }
    //  }

    fn get_one(&self, query: Query) -> Result<(Cid, Delegation<T, C>), Self::Error> {
        todo!()
        //let mut results = Self::get_by(query)?;
        //results.pop().ok_or_else(|_| todo!())
    }

    fn expired(&self) -> Result<BTreeMap<Cid, Delegation<T, C>>, Self::Error> {
        todo!()
        // self.get_by(Query {
        //     expires_before: Some(SystemTime::now()),
        //     ..Default::default()
        // })
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Query {
    pub subject: Option<Did>,
    pub command: Option<String>,
    pub issuer: Option<Did>,
    pub audience: Option<Did>,

    pub prior_to_not_before: Option<SystemTime>, // FIXME time
    pub after_not_before: Option<SystemTime>,    // FIXME time

    pub expires_before: Option<SystemTime>, // FIXME time
    pub expires_aftre: Option<SystemTime>,  // FIXME time
}
