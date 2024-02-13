use super::{condition::Condition, Delegation};
use crate::{
    did::Did,
    proof::{checkable::Checkable, prove::Prove},
};
use libipld_core::cid::Cid;
use nonempty::NonEmpty;
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::Infallible,
    ops::ControlFlow,
};
use web_time::SystemTime;

// NOTE the T here is the builder... FIXME add one layer up and call T::Builder? May be confusing?
pub trait Store<B: Checkable, C: Condition, DID: Did> {
    type Error;

    fn get(&self, cid: &Cid) -> Result<Option<&Delegation<B::Hierarchy, C, DID>>, Self::Error>;

    // FIXME add a variant that calculated the CID from the capsulre header?
    // FIXME that means changing the name to insert_by_cid or similar
    fn insert(&mut self, cid: Cid, delegation: Delegation<B, C, DID>) -> Result<(), Self::Error>;

    fn revoke(&mut self, cid: Cid) -> Result<(), Self::Error>;

    fn get_chain(
        &self,
        aud: &DID,
        subject: &DID,
        builder: &B,
        now: &SystemTime,
    ) -> Result<Option<NonEmpty<(&Cid, &Delegation<B::Hierarchy, C, DID>)>>, Self::Error>;

    fn can_delegate(
        &self,
        iss: &DID,
        aud: &DID,
        builder: &B,
        now: &SystemTime,
    ) -> Result<bool, Self::Error> {
        self.get_chain(aud, iss, builder, now)
            .map(|chain| chain.is_some())
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
pub struct MemoryStore<H, C: Condition, DID: Did + Ord> {
    ucans: BTreeMap<Cid, Delegation<H, C, DID>>,
    index: BTreeMap<DID, BTreeMap<DID, BTreeSet<Cid>>>,
    revocations: BTreeSet<Cid>,
}

// FIXME check that UCAN is valid
impl<B: Checkable + Clone, C: Condition + PartialEq, DID: Did + Ord + Clone> Store<B, C, DID>
    for MemoryStore<B::Hierarchy, C, DID>
where
    B::Hierarchy: PartialEq,
{
    type Error = Infallible;

    fn get(&self, cid: &Cid) -> Result<Option<&Delegation<B::Hierarchy, C, DID>>, Self::Error> {
        Ok(self.ucans.get(cid))
    }

    fn insert(&mut self, cid: Cid, delegation: Delegation<B, C, DID>) -> Result<(), Self::Error> {
        self.index
            .entry(delegation.payload.subject.clone())
            .or_default()
            .entry(delegation.payload.audience.clone())
            .or_default()
            .insert(cid);

        let hierarchy: Delegation<B::Hierarchy, C, DID> = Delegation {
            signature: delegation.signature,
            payload: delegation.payload.map_ability(Into::into),
        };

        self.ucans.insert(cid.clone(), hierarchy);
        Ok(())
    }

    fn revoke(&mut self, cid: Cid) -> Result<(), Self::Error> {
        self.revocations.insert(cid);
        Ok(())
    }

    fn get_chain(
        &self,
        aud: &DID,
        subject: &DID,
        builder: &B,
        now: &SystemTime,
    ) -> Result<Option<NonEmpty<(&Cid, &Delegation<B::Hierarchy, C, DID>)>>, Self::Error> {
        match self.index.get(subject).and_then(|aud_map| aud_map.get(aud)) {
            None => Ok(None),
            Some(delegation_subtree) => {
                #[derive(PartialEq)]
                enum Status {
                    Complete,
                    Looking,
                    NoPath,
                }

                let mut status = Status::Looking;
                let mut target_aud = aud;
                let mut args = &B::Hierarchy::from(builder.clone());
                let mut chain = vec![];

                while status == Status::Looking {
                    let found = delegation_subtree.iter().try_for_each(|cid| {
                        if let Some(d) = self.ucans.get(cid) {
                            if self.revocations.contains(cid) {
                                return ControlFlow::Continue(());
                            }

                            if d.payload.check_time(*now).is_err() {
                                return ControlFlow::Continue(());
                            }

                            target_aud = &d.payload.audience;

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
                    Status::Complete => Ok(NonEmpty::from_vec(chain)),
                    _ => Ok(None),
                }
            }
        }
    }
}
