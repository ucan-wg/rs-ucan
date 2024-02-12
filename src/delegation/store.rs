use super::{condition::Condition, delegatable::Delegatable, Delegation};
use crate::did::Did;
use libipld_core::cid::Cid;
use nonempty::NonEmpty;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::ControlFlow,
};
use thiserror::Error;
use web_time::SystemTime;

pub trait Store<T, C: Condition> {
    type Error;

    fn get_by_cid(&self, cid: &Cid) -> Result<&Delegation<T, C>, Self::Error>;

    fn insert(&mut self, cid: &Cid, delegation: Delegation<T, C>);
    fn revoke(&mut self, cid: &Cid);

    fn get_chain(
        &self,
        aud: &Did,
        subject: &Did,
        now: &SystemTime,
    ) -> Result<NonEmpty<(&Cid, &Delegation<T, C>)>, Self::Error>;
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// A simple in-memory store for delegations.
///
/// The store is laid out as follows:
///
/// `{Subject => {Audience => {Cid => Delegation}}}`
///
/// ```mermaid
/// flowchart LR
/// subgraph Subjects
///     direction TB
///
///     Akiko
///     Boris
///     Carol
///
///     subgraph aud[Boris's Audiences]
///         direction TB
///
///         Denzel
///         Erin
///         Frida
///         Georgia
///         Hugo
///
///         subgraph cid[Frida's CIDs]
///             direction LR
///
///             CID1 --> Delegation1
///             CID2 --> Delegation2
///             CID3 --> Delegation3
///         end
///     end
/// end
///
/// Akiko ~~~ Hugo
/// Carol ~~~ Hugo
/// Boris --> Frida --> CID2
///
/// Boris -.-> Denzel
/// Boris -.-> Erin
/// Boris -.-> Georgia
/// Boris -.-> Hugo
///
/// Frida -.-> CID1
/// Frida -.-> CID3
///
/// style Boris stroke:orange;
/// style Frida stroke:orange;
/// style CID2 stroke:orange;
/// style Delegation2 stroke:orange;
///
/// linkStyle 5 stroke:orange;
/// linkStyle 6 stroke:orange;
/// linkStyle 1 stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStore<T, C: Condition> {
    ucans: BTreeMap<Cid, Delegation<T, C>>,
    index: BTreeMap<Did, BTreeMap<Did, BTreeSet<Cid>>>,
    revocations: BTreeSet<Cid>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
#[error("Delegation not found")]
pub struct NotFound;

// FIXME check that UCAN is valid
impl<T, C: Condition> Store<T, C> for MemoryStore<T, C> {
    type Error = NotFound;

    fn insert(&mut self, cid: &Cid, delegation: Delegation<T, C>) {
        self.index
            .entry(delegation.payload.subject.clone())
            .or_default()
            .entry(delegation.payload.audience.clone())
            .or_default()
            .insert(cid.clone());

        self.ucans.insert(cid.clone(), delegation);
    }

    fn revoke(&mut self, cid: &Cid) {
        self.revocations.insert(cid.clone());
    }

    fn get_by_cid(&self, cid: &Cid) -> Result<&Delegation<T, C>, Self::Error> {
        self.ucans.get(cid).ok_or(NotFound)
    }

    fn get_chain(
        &self,
        aud: &Did,
        subject: &Did,
        now: &SystemTime,
    ) -> Result<NonEmpty<(&Cid, &Delegation<T, C>)>, NotFound> {
        #[derive(PartialEq)]
        enum Status {
            Complete,
            Looking,
            NoPath,
        }

        let mut status = Status::Looking;
        let mut target_aud = aud;
        let mut chain = vec![];

        let delegation_subtree = self
            .index
            .get(subject)
            .and_then(|aud_map| aud_map.get(aud))
            .ok_or(NotFound)?;

        while status == Status::Looking {
            let found = delegation_subtree.iter().try_for_each(|cid| {
                if self.revocations.contains(&cid) {
                    return ControlFlow::Continue(());
                }

                if let Some(d) = self.ucans.get(cid) {
                    if SystemTime::from(d.payload.expiration.clone()) < *now {
                        return ControlFlow::Continue(());
                    }

                    if let Some(nbf) = &d.payload.not_before {
                        if SystemTime::from(nbf.clone()) > *now {
                            return ControlFlow::Continue(());
                        }
                    }

                    chain.push((cid, d));

                    if &d.payload.issuer == subject {
                        status = Status::Complete;
                    } else {
                        target_aud = &d.payload.issuer;
                    }

                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            });

            if found.is_continue() {
                status = Status::NoPath;
            }
        }

        match status {
            Status::Complete => NonEmpty::from_vec(chain).ok_or(NotFound),
            _ => Err(NotFound),
        }
    }
}
