use super::{condition::Condition, Delegable, Delegation};
use crate::{
    did::Did,
    proof::{checkable::Checkable, prove::Prove},
};
use libipld_core::cid::Cid;
use nonempty::NonEmpty;
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::ControlFlow,
};
use thiserror::Error;
use web_time::SystemTime;

// NOTE the T here is the builder... FIXME add one layer up and call T::Builder? May be confusing?
pub trait Store<B: Checkable, C: Condition> {
    type Error;

    fn get(&self, cid: &Cid) -> Result<&Delegation<B::Hierarchy, C>, Self::Error>;

    fn insert(&mut self, cid: Cid, delegation: Delegation<B, C>);

    fn revoke(&mut self, cid: Cid);

    fn get_chain(
        &self,
        aud: &Did,
        subject: &Did,
        builder: &B,
        now: &SystemTime,
    ) -> Result<NonEmpty<(&Cid, &Delegation<B::Hierarchy, C>)>, Self::Error>;

    fn can_delegate(&self, iss: &Did, aud: &Did, builder: &B, now: &SystemTime) -> bool {
        self.get_chain(aud, iss, builder, now).is_ok()
    }
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
pub struct MemoryStore<H, C: Condition> {
    ucans: BTreeMap<Cid, Delegation<H, C>>,
    index: BTreeMap<Did, BTreeMap<Did, BTreeSet<Cid>>>,
    revocations: BTreeSet<Cid>,
}

// FIXME extract
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
#[error("Delegation not found")]
pub struct NotFound;

// FIXME check that UCAN is valid
impl<B: Checkable + Clone, C: Condition> Store<B, C> for MemoryStore<B::Hierarchy, C> {
    type Error = NotFound;

    fn get(&self, cid: &Cid) -> Result<&Delegation<B::Hierarchy, C>, Self::Error> {
        self.ucans.get(cid).ok_or(NotFound)
    }

    fn insert(&mut self, cid: Cid, delegation: Delegation<B, C>) {
        self.index
            .entry(delegation.payload.subject.clone())
            .or_default()
            .entry(delegation.payload.audience.clone())
            .or_default()
            .insert(cid);

        let hierarchy: Delegation<B::Hierarchy, C> = Delegation {
            signature: delegation.signature,
            payload: delegation.payload.map_ability(Into::into),
        };

        self.ucans.insert(cid.clone(), hierarchy);
    }

    fn revoke(&mut self, cid: Cid) {
        self.revocations.insert(cid);
    }

    fn get_chain(
        &self,
        aud: &Did,
        subject: &Did,
        builder: &B,
        now: &SystemTime,
    ) -> Result<NonEmpty<(&Cid, &Delegation<B::Hierarchy, C>)>, NotFound> {
        #[derive(PartialEq)]
        enum Status {
            Complete,
            Looking,
            NoPath,
        }

        // FIXME move these into an Acc
        let mut status = Status::Looking;
        let mut target_aud = aud;
        let mut chain = vec![];
        let mut args: &B::Hierarchy = &builder.clone().into();

        let delegation_subtree = self
            .index
            .get(subject)
            .and_then(|aud_map| aud_map.get(aud))
            .ok_or(NotFound)?;

        while status == Status::Looking {
            let found = delegation_subtree.iter().try_for_each(|cid| {
                if let Some(d) = self.ucans.get(cid) {
                    if self.revocations.contains(&cid) {
                        return ControlFlow::Continue(());
                    }

                    if d.payload.check_time(*now).is_err() {
                        return ControlFlow::Continue(());
                    }

                    if args.check(&d.payload.ability_builder).is_ok() {
                        args = &d.payload.ability_builder;
                    } else {
                        return ControlFlow::Continue(());
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
