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
        self.read().ucans.is_empty() // FIXME account for revocations?
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
    Ipld: Encode<Enc>,
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

    fn insert_keyed(
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
        subject: &DID,
        command: String,
        policy: Vec<Predicate>,
        now: SystemTime,
    ) -> Result<Option<NonEmpty<(Cid, Arc<Delegation<DID, V, Enc>>)>>, Self::DelegationStoreError>
    {
        let blank_set = BTreeSet::new();
        let blank_map = BTreeMap::new();
        let read_tx = self.read();

        let all_powerlines = read_tx.index.get(&None).unwrap_or(&blank_map);
        let all_aud_for_subject = read_tx
            .index
            .get(&Some(subject.clone()))
            .unwrap_or(&blank_map);
        let powerline_candidates = all_powerlines.get(aud).unwrap_or(&blank_set);
        let sub_candidates = all_aud_for_subject.get(aud).unwrap_or(&blank_set);

        let mut parent_candidate_stack =
            vec![sub_candidates.iter().chain(powerline_candidates.iter())];
        let mut hypothesis_chain = vec![];

        let corrected_target_command = if command.ends_with('/') {
            command
        } else {
            format!("{}/", command)
        };

        'outer: loop {
            if let Some(parent_cid_candidates) = parent_candidate_stack.last_mut() {
                if parent_cid_candidates.clone().collect::<Vec<_>>().is_empty() {
                    parent_candidate_stack.pop();
                    continue;
                }

                'inner: for cid in parent_cid_candidates {
                    // CHECKS
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

                        if !corrected_target_command.starts_with(&corrected_delegation_command) {
                            continue;
                        }

                        // FIXME
                        // for target_pred in policy.iter() {
                        //     for delegate_pred in delegation.payload.policy.iter() {
                        //         let comparison =
                        //             target_pred.harmonize(delegate_pred, vec![], vec![]);

                        //         if comparison.is_conflict() || comparison.is_lhs_weaker() {
                        //             continue 'inner;
                        //         }
                        //     }
                        // }

                        // PASSED CHECKS, so processing
                        hypothesis_chain.push((cid.clone(), Arc::clone(delegation)));

                        let issuer = delegation.issuer().clone();

                        // Hit a root delegation, AKA base case
                        if &Some(issuer.clone()) == delegation.subject() {
                            break 'outer;
                        }

                        let new_aud_candidates =
                            all_aud_for_subject.get(&issuer).unwrap_or(&blank_set);

                        if !new_aud_candidates.is_empty() || !all_powerlines.get(&issuer).is_none()
                        {
                            parent_candidate_stack.push(
                                new_aud_candidates.iter().chain(
                                    all_powerlines.get(&issuer).unwrap_or(&blank_set).iter(),
                                ),
                            );

                            break 'inner;
                        }
                    }
                }
            } else {
                parent_candidate_stack.pop();
                break 'outer;
            }
        }

        Ok(NonEmpty::from_vec(hypothesis_chain))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::varsig::encoding;
    use crate::crypto::varsig::header;
    use crate::{crypto::signature::Envelope, delegation::store::Store};

    use libipld_core::cid::Cid;
    use nonempty::nonempty;
    use pretty_assertions as pretty;
    use rand::thread_rng;
    use std::time::SystemTime;
    use testresult::TestResult;

    fn gen_did() -> (crate::did::preset::Verifier, crate::did::preset::Signer) {
        let sk = ed25519_dalek::SigningKey::generate(&mut thread_rng());
        let verifier =
            crate::did::preset::Verifier::Key(crate::did::key::Verifier::EdDsa(sk.verifying_key()));
        let signer = crate::did::preset::Signer::Key(crate::did::key::Signer::EdDsa(sk));

        (verifier, signer)
    }

    #[test_log::test]
    fn test_get_fail() -> TestResult {
        let store = MemoryStore::<
            did::preset::Verifier,
            varsig::header::Preset,
            varsig::encoding::Preset,
        >::default();
        store.get(&Cid::default())?;
        pretty::assert_eq!(store.get(&Cid::default()), Ok(None));
        Ok(())
    }

    #[test_log::test]
    fn test_insert_get_roundtrip() -> TestResult {
        let (did, signer) = gen_did();

        let store = MemoryStore::default();
        let varsig_header = header::Preset::EdDsa(header::EdDsaHeader {
            codec: encoding::Preset::DagCbor,
        });

        let deleg = Delegation::try_sign(
            &signer,
            varsig_header,
            crate::delegation::PayloadBuilder::default()
                .subject(None)
                .issuer(did.clone())
                .audience(did.clone())
                .command("/".into())
                .expiration(crate::time::Timestamp::five_years_from_now())
                .build()?,
        )?;

        store.insert(deleg.clone())?;
        let retrieved = store.get(&deleg.cid()?)?.ok_or("failed to retrieve")?;

        pretty::assert_eq!(deleg, *retrieved);

        Ok(())
    }

    #[test_log::test]
    fn test_insert_is_idempotent() -> TestResult {
        let (did, signer) = gen_did();

        let store = MemoryStore::default();
        let varsig_header = header::Preset::EdDsa(header::EdDsaHeader {
            codec: encoding::Preset::DagCbor,
        });

        let deleg = Delegation::try_sign(
            &signer,
            varsig_header,
            crate::delegation::PayloadBuilder::default()
                .subject(None)
                .issuer(did.clone())
                .audience(did.clone())
                .command("/".into())
                .expiration(crate::time::Timestamp::five_years_from_now())
                .build()?,
        )?;

        store.insert(deleg.clone())?;
        store.insert(deleg.clone())?;
        store.insert(deleg.clone())?;
        store.insert(deleg.clone())?;
        store.insert(deleg.clone())?;
        store.insert(deleg.clone())?;
        store.insert(deleg.clone())?;

        let retrieved = store.get(&deleg.cid()?)?.ok_or("failed to retrieve")?;

        pretty::assert_eq!(deleg, *retrieved);
        pretty::assert_eq!(store.len(), 1);

        Ok(())
    }

    mod get_chain {
        use super::*;

        #[test_log::test]
        fn test_simple_fail() -> TestResult {
            let (server, _server_signer) = gen_did();
            let (nope, _nope_signer) = gen_did();

            let store = MemoryStore::<
                did::preset::Verifier,
                varsig::header::Preset,
                varsig::encoding::Preset,
            >::default();
            let got = store.get_chain(&server, &nope, "/".into(), vec![], SystemTime::now())?;

            pretty::assert_eq!(got, None);
            Ok(())
        }

        #[test_log::test]
        fn test_with_one() -> TestResult {
            let (alice, alice_signer) = gen_did();
            let (bob, _bob_signer) = gen_did();

            let store = crate::delegation::store::MemoryStore::default();
            let varsig_header = crate::crypto::varsig::header::Preset::EdDsa(
                crate::crypto::varsig::header::EdDsaHeader {
                    codec: crate::crypto::varsig::encoding::Preset::DagCbor,
                },
            );

            let deleg = crate::Delegation::try_sign(
                &alice_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(None)
                    .issuer(alice.clone())
                    .audience(bob.clone())
                    .command("/".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            store.insert(deleg.clone())?;

            let got = store.get_chain(&bob, &alice, "/".into(), vec![], SystemTime::now())?;
            pretty::assert_eq!(got, Some(nonempty![(deleg.cid()?, Arc::new(deleg))].into()));
            Ok(())
        }

        #[test_log::test]
        fn test_with_one_with_others_in_store() -> TestResult {
            let (alice, alice_signer) = gen_did();
            let (bob, bob_signer) = gen_did();
            let (carol, _carol_signer) = gen_did();

            let store = crate::delegation::store::MemoryStore::default();
            let varsig_header = crate::crypto::varsig::header::Preset::EdDsa(
                crate::crypto::varsig::header::EdDsaHeader {
                    codec: crate::crypto::varsig::encoding::Preset::DagCbor,
                },
            );

            let noise = crate::Delegation::try_sign(
                &bob_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(None)
                    .issuer(bob.clone())
                    .audience(carol.clone())
                    .command("/example".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            store.insert(noise.clone())?;

            let deleg = crate::Delegation::try_sign(
                &alice_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(None)
                    .issuer(alice.clone())
                    .audience(bob.clone())
                    .command("/".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            store.insert(deleg.clone())?;

            let more_noise = crate::Delegation::try_sign(
                &alice_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(None)
                    .issuer(alice.clone())
                    .audience(carol.clone())
                    .command("/test".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            store.insert(more_noise.clone())?;

            let got = store.get_chain(&bob, &alice, "/".into(), vec![], SystemTime::now())?;
            pretty::assert_eq!(got, Some(nonempty![(deleg.cid()?, Arc::new(deleg))].into()));
            Ok(())
        }

        #[test_log::test]
        fn test_with_two() -> TestResult {
            let (alice, alice_signer) = gen_did();
            let (bob, bob_signer) = gen_did();
            let (carol, _carol_signer) = gen_did();

            let store = crate::delegation::store::MemoryStore::default();
            let varsig_header = crate::crypto::varsig::header::Preset::EdDsa(
                crate::crypto::varsig::header::EdDsaHeader {
                    codec: crate::crypto::varsig::encoding::Preset::DagCbor,
                },
            );

            let deleg_1 = crate::Delegation::try_sign(
                &alice_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(None)
                    .issuer(alice.clone())
                    .audience(bob.clone())
                    .command("/".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            store.insert(deleg_1.clone())?;

            let deleg_2 = crate::Delegation::try_sign(
                &bob_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(Some(alice.clone()))
                    .issuer(bob.clone())
                    .audience(carol.clone())
                    .command("/".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            store.insert(deleg_2.clone())?;

            let got = store.get_chain(&carol, &alice, "/".into(), vec![], SystemTime::now())?;

            pretty::assert_eq!(
                got,
                Some(
                    nonempty![
                        (deleg_2.cid()?, Arc::new(deleg_2)),
                        (deleg_1.cid()?, Arc::new(deleg_1)),
                    ]
                    .into()
                )
            );
            Ok(())
        }

        #[test_log::test]
        fn test_looking_for_narrower_command() -> TestResult {
            let (alice, alice_signer) = gen_did();
            let (bob, bob_signer) = gen_did();
            let (carol, _carol_signer) = gen_did();

            let store = crate::delegation::store::MemoryStore::default();
            let varsig_header = crate::crypto::varsig::header::Preset::EdDsa(
                crate::crypto::varsig::header::EdDsaHeader {
                    codec: crate::crypto::varsig::encoding::Preset::DagCbor,
                },
            );

            let deleg_1 = crate::Delegation::try_sign(
                &alice_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(None)
                    .issuer(alice.clone())
                    .audience(bob.clone())
                    .command("/test".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            store.insert(deleg_1.clone())?;

            let deleg_2 = crate::Delegation::try_sign(
                &bob_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(Some(alice.clone()))
                    .issuer(bob.clone())
                    .audience(carol.clone())
                    .command("/test/me".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            store.insert(deleg_2.clone())?;

            let got = store.get_chain(
                &carol,
                &alice,
                "/test/me/now".into(),
                vec![],
                SystemTime::now(),
            )?;

            pretty::assert_eq!(
                got,
                Some(
                    nonempty![
                        (deleg_2.cid()?, Arc::new(deleg_2)),
                        (deleg_1.cid()?, Arc::new(deleg_1)),
                    ]
                    .into()
                )
            );
            Ok(())
        }

        #[test_log::test]
        fn test_broken_chain() -> TestResult {
            let (alice, alice_signer) = gen_did();
            let (bob, _bob_signer) = gen_did();
            let (carol, carol_signer) = gen_did();
            let (dan, _dan_signer) = gen_did();

            let store = crate::delegation::store::MemoryStore::default();
            let varsig_header = crate::crypto::varsig::header::Preset::EdDsa(
                crate::crypto::varsig::header::EdDsaHeader {
                    codec: crate::crypto::varsig::encoding::Preset::DagCbor,
                },
            );

            let alice_to_bob = crate::Delegation::try_sign(
                &alice_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(None)
                    .issuer(alice.clone())
                    .audience(bob.clone())
                    .command("/test".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            store.insert(alice_to_bob.clone())?;

            let carol_to_dan = crate::Delegation::try_sign(
                &carol_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(Some(alice.clone()))
                    .issuer(carol.clone())
                    .audience(dan.clone())
                    .command("/test/me".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            store.insert(carol_to_dan.clone())?;

            let got = store.get_chain(
                &carol,
                &alice,
                "/test/me/now".into(),
                vec![],
                SystemTime::now(),
            )?;

            pretty::assert_eq!(got, None);
            Ok(())
        }

        #[test_log::test]
        fn test_long_chain() -> TestResult {
            // Scenario
            // ========
            // 1.             bob -*-> carol
            // 2.                      carol -a-> dave
            // 3.  alice -d-> bob
            let (alice, alice_signer) = gen_did();
            let (bob, bob_signer) = gen_did();
            let (carol, carol_signer) = gen_did();
            let (dave, _) = gen_did();

            let varsig_header = crate::crypto::varsig::header::Preset::EdDsa(
                crate::crypto::varsig::header::EdDsaHeader {
                    codec: crate::crypto::varsig::encoding::Preset::DagCbor,
                },
            );

            let store = crate::delegation::store::MemoryStore::default();

            // 1.               bob -*-> carol
            let bob_to_carol = crate::Delegation::try_sign(
                &bob_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(None)
                    .issuer(bob.clone())
                    .audience(carol.clone())
                    .command("/".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            // 2.                      carol -a-> dave
            let carol_to_dave = crate::Delegation::try_sign(
                &carol_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(None)
                    .issuer(carol.clone())
                    .audience(dave.clone())
                    .command("/".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?, // I don't love this is now failable
            )?;

            // 3.  alice -d-> bob
            let alice_to_bob = crate::Delegation::try_sign(
                &alice_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(Some(alice.clone()))
                    .issuer(alice.clone())
                    .audience(bob.clone())
                    .command("/".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            store.insert(bob_to_carol.clone())?;
            store.insert(carol_to_dave.clone())?;
            store.insert(alice_to_bob.clone())?;

            let got: Vec<Cid> = store
                .get_chain(&dave, &alice, "/".into(), vec![], SystemTime::now())
                .map_err(|e| e.to_string())?
                .ok_or("failed during proof lookup")?
                .iter()
                .map(|(cid, _)| cid)
                .cloned()
                .collect();

            pretty::assert_eq!(
                got,
                vec![
                    carol_to_dave.cid()?,
                    bob_to_carol.cid()?,
                    alice_to_bob.cid()?
                ]
            );

            Ok(())
        }

        #[test_log::test]
        fn test_long_powerline() -> TestResult {
            // Scenario
            // ========
            // 1.             bob -*-> carol
            // 2.                      carol -a-> dave
            // 3.  alice -d-> bob
            let (alice, alice_signer) = gen_did();
            let (bob, bob_signer) = gen_did();
            let (carol, carol_signer) = gen_did();
            let (dave, _) = gen_did();

            let varsig_header = crate::crypto::varsig::header::Preset::EdDsa(
                crate::crypto::varsig::header::EdDsaHeader {
                    codec: crate::crypto::varsig::encoding::Preset::DagCbor,
                },
            );

            let store = crate::delegation::store::MemoryStore::default();

            // 1.               bob -*-> carol
            let bob_to_carol = crate::Delegation::try_sign(
                &bob_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(None)
                    .issuer(bob.clone())
                    .audience(carol.clone())
                    .command("/".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            // 2.                      carol -a-> dave
            let carol_to_dave = crate::Delegation::try_sign(
                &carol_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(None)
                    .issuer(carol.clone())
                    .audience(dave.clone())
                    .command("/".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?, // I don't love this is now failable
            )?;

            // 3.  alice -d-> bob
            let alice_to_bob = crate::Delegation::try_sign(
                &alice_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(Some(alice.clone()))
                    .issuer(alice.clone())
                    .audience(bob.clone())
                    .command("/".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            store.insert(bob_to_carol.clone())?;
            store.insert(carol_to_dave.clone())?;
            store.insert(alice_to_bob.clone())?;

            let got: Vec<Cid> = store
                .get_chain(&dave, &alice.clone(), "/".into(), vec![], SystemTime::now())
                .map_err(|e| e.to_string())?
                .ok_or("failed during proof lookup")?
                .iter()
                .map(|(cid, _)| cid)
                .cloned()
                .collect();

            pretty::assert_eq!(
                got,
                vec![
                    carol_to_dave.cid()?,
                    bob_to_carol.cid()?,
                    alice_to_bob.cid()?
                ]
            );

            Ok(())
        }
    }
}
