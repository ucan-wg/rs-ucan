use super::Store;
use crate::ability::arguments::Named;
use crate::delegation;
use crate::{
    crypto::varsig,
    delegation::{policy::Predicate, Delegation},
    did::{self, Did},
};
use libipld_core::codec::Encode;
use libipld_core::ipld::Ipld;
use libipld_core::{cid::Cid, codec::Codec};
use nonempty::NonEmpty;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::Infallible,
};
use web_time::SystemTime;

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
#[derive(Debug, Clone)]
pub struct MemoryStore<
    DID: did::Did + Ord = did::preset::Verifier,
    V: varsig::Header<C> = varsig::header::Preset,
    C: Codec + TryFrom<u64> + Into<u64> = varsig::encoding::Preset,
> {
    inner: Arc<RwLock<MemoryStoreInner<DID, V, C>>>,
}

#[derive(Debug, Clone, PartialEq)]
struct MemoryStoreInner<
    DID: did::Did + Ord = did::preset::Verifier,
    V: varsig::Header<C> = varsig::header::Preset,
    C: Codec + TryFrom<u64> + Into<u64> = varsig::encoding::Preset,
> {
    ucans: BTreeMap<Cid, Arc<Delegation<DID, V, C>>>,
    index: BTreeMap<Option<DID>, BTreeMap<DID, BTreeSet<Cid>>>,
    revocations: BTreeSet<Cid>,
}

impl<DID: did::Did + Ord, V: varsig::Header<C>, C: Codec + TryFrom<u64> + Into<u64>>
    MemoryStore<DID, V, C>
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.read().ucans.len()
    }

    pub fn is_empty(&self) -> bool {
        self.read().ucans.is_empty() // FIXME acocunt for revocations?
    }

    fn read(&self) -> RwLockReadGuard<'_, MemoryStoreInner<DID, V, C>> {
        match self.inner.read() {
            Ok(guard) => guard,
            Err(poison) => {
                // We ignore lock poisoning for simplicity
                poison.into_inner()
            }
        }
    }

    fn write(&self) -> RwLockWriteGuard<'_, MemoryStoreInner<DID, V, C>> {
        match self.inner.write() {
            Ok(guard) => guard,
            Err(poison) => {
                // We ignore lock poisoning for simplicity
                poison.into_inner()
            }
        }
    }
}

impl<DID: did::Did + Ord, V: varsig::Header<C>, C: Codec + TryFrom<u64> + Into<u64>> Default
    for MemoryStore<DID, V, C>
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<DID: Did + Ord, V: varsig::Header<C>, C: Codec + TryFrom<u64> + Into<u64>> Default
    for MemoryStoreInner<DID, V, C>
{
    fn default() -> Self {
        MemoryStoreInner {
            ucans: BTreeMap::new(),
            index: BTreeMap::new(),
            revocations: BTreeSet::new(),
        }
    }
}

// FIXME check that UCAN is valid
impl<
        DID: Did + Ord + Clone,
        V: varsig::Header<Enc> + Clone,
        Enc: Codec + TryFrom<u64> + Into<u64>,
    > Store<DID, V, Enc> for MemoryStore<DID, V, Enc>
where
    Named<Ipld>: From<delegation::Payload<DID>>,
    delegation::Payload<DID>: TryFrom<Named<Ipld>>,
    Delegation<DID, V, Enc>: Encode<Enc>,
{
    type DelegationStoreError = Infallible;

    fn get(
        &self,
        cid: &Cid,
    ) -> Result<Option<Arc<Delegation<DID, V, Enc>>>, Self::DelegationStoreError> {
        // cheap Arc clone
        Ok(self.read().ucans.get(cid).cloned())
        // FIXME
    }

    fn insert(
        &self,
        cid: Cid,
        delegation: Delegation<DID, V, Enc>,
    ) -> Result<(), Self::DelegationStoreError> {
        let mut write_tx = self.write();

        write_tx
            .index
            .entry(delegation.subject().clone())
            .or_default()
            .entry(delegation.audience().clone())
            .or_default()
            .insert(cid);

        write_tx.ucans.insert(cid.clone(), Arc::new(delegation));

        Ok(())
    }

    fn revoke(&self, cid: Cid) -> Result<(), Self::DelegationStoreError> {
        self.write().revocations.insert(cid);
        Ok(())
    }

    fn get_chain(
        &self,
        aud: &DID,
        subject: &Option<DID>,
        command: String,
        policy: Vec<Predicate>, // FIXME
        now: SystemTime,
    ) -> Result<Option<NonEmpty<(Cid, Arc<Delegation<DID, V, Enc>>)>>, Self::DelegationStoreError>
    {
        let blank_set = BTreeSet::new();
        let blank_map = BTreeMap::new();
        let read_tx = self.read();

        let all_powerlines = read_tx.index.get(&None).unwrap_or(&blank_map);
        let all_aud_for_subject = read_tx.index.get(subject).unwrap_or(&blank_map);
        let powerline_candidates = all_powerlines.get(aud).unwrap_or(&blank_set);
        let sub_candidates = all_aud_for_subject.get(aud).unwrap_or(&blank_set);

        let mut parent_candidate_stack = vec![];
        let mut hypothesis_chain = vec![];

        let corrected_target_command = if command.ends_with('/') {
            command
        } else {
            format!("{}/", command)
        };

        parent_candidate_stack.push(sub_candidates.iter().chain(powerline_candidates.iter()));
        let mut next = None;

        'outer: loop {
            if let Some(parent_cid_candidates) = parent_candidate_stack.last_mut() {
                if parent_cid_candidates.clone().collect::<Vec<_>>().is_empty() {
                    parent_candidate_stack.pop();
                    hypothesis_chain.pop();
                    break 'outer;
                }

                'inner: for cid in parent_cid_candidates {
                    if read_tx.revocations.contains(cid) {
                        continue;
                    }

                    if let Some(delegation) = read_tx.ucans.get(cid) {
                        if delegation.check_time(now).is_err() {
                            continue;
                        }

                        // FIXME extract
                        let corrected_delegation_command =
                            if delegation.payload.command.ends_with('/') {
                                delegation.payload.command.clone()
                            } else {
                                format!("{}/", delegation.payload.command)
                            };

                        if !corrected_delegation_command.starts_with(&corrected_target_command) {
                            continue;
                        }

                        for target_pred in policy.iter() {
                            for delegate_pred in delegation.payload.policy.iter() {
                                let comparison =
                                    target_pred.harmonize(delegate_pred, vec![], vec![]);

                                if comparison.is_conflict() || comparison.is_lhs_weaker() {
                                    continue 'inner;
                                }
                            }
                        }

                        hypothesis_chain.push((cid.clone(), Arc::clone(delegation)));

                        let issuer = delegation.issuer().clone();

                        // Hit a root delegation, AKA base case
                        if &Some(issuer.clone()) == delegation.subject() {
                            break 'outer;
                        }

                        let new_aud_candidates =
                            all_aud_for_subject.get(&issuer).unwrap_or(&blank_set);

                        let new_powerline_candidates =
                            all_powerlines.get(&issuer).unwrap_or(&blank_set);

                        if !new_aud_candidates.is_empty() || !new_powerline_candidates.is_empty() {
                            next = Some(
                                new_aud_candidates
                                    .iter()
                                    .chain(new_powerline_candidates.iter()),
                            );

                            break 'inner;
                        }
                    }
                }

                if let Some(ref n) = next {
                    parent_candidate_stack.push(n.clone());
                    next = None;
                } else {
                    // Didn't find a match
                    break 'outer;
                }
            } else {
                parent_candidate_stack.pop();
                hypothesis_chain.pop();
            }
        }

        Ok(NonEmpty::from_vec(hypothesis_chain))
    }
}
