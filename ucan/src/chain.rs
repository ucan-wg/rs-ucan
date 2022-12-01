use crate::{
    capability::{
        proof::{ProofDelegationSemantics, ProofSelection},
        Action, Capability, CapabilityIterator, CapabilitySemantics, Resource, Scope, With,
    },
    crypto::did::DidParser,
    store::UcanJwtStore,
    ucan::Ucan,
};
use anyhow::{anyhow, Context, Result};
use cid::Cid;
#[cfg(not(target_arch = "wasm32"))]
use futures::future::{BoxFuture, FutureExt};
#[cfg(target_arch = "wasm32")]
use futures::future::{FutureExt, LocalBoxFuture};
use std::{
    collections::BTreeSet,
    convert::TryFrom,
    fmt::{self, Debug},
    str::FromStr,
    sync::Arc,
};

/// Type alias for handling recursive async fns, akin to async_recursion crate.
#[cfg(not(target_arch = "wasm32"))]
type BuildResult<'a, S> = BoxFuture<'a, Result<ProofChain<'a, S>>>;

/// Type alias for handling recursive async fns in wasm, akin to async_recursion crate.
#[cfg(target_arch = "wasm32")]
type BuildResult<'a, S> = LocalBoxFuture<'a, Result<ProofChain<'a, S>>>;

const PROOF_DELEGATION_SEMANTICS: ProofDelegationSemantics = ProofDelegationSemantics {};

#[derive(Eq, PartialEq)]
pub struct CapabilityInfo<S: Scope, A: Action> {
    pub originators: BTreeSet<String>,
    pub not_before: Option<u64>,
    pub expires_at: u64,
    pub capability: Capability<S, A>,
}

impl<S, A> Debug for CapabilityInfo<S, A>
where
    S: Scope,
    A: Action,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapabilityInfo")
            .field("originators", &self.originators)
            .field("not_before", &self.not_before)
            .field("expires_at", &self.expires_at)
            .field("capability", &self.capability)
            .finish()
    }
}

/// A deserialized chain of ancestral proofs that are linked to a UCAN
pub struct ProofChain<'a, S>
where
    S: UcanJwtStore,
{
    ucan: Ucan,
    proofs: Vec<ProofChain<'a, S>>,
    redelegations: BTreeSet<usize>,
    did_parser: Option<Arc<DidParser>>,
    store: Option<&'a S>,
}

impl<S> Debug for ProofChain<'_, S>
where
    S: UcanJwtStore,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProofChain")
            .field("ucan", &self.ucan)
            .field("proofs", &self.proofs)
            .field("redelegations", &self.redelegations)
            .finish()
    }
}

impl<'a, S> ProofChain<'a, S>
where
    S: UcanJwtStore,
{
    /// Instantiate a bare [ProofChain] from a [Ucan].
    pub fn from_ucan(ucan: Ucan) -> Self {
        Self {
            ucan,
            proofs: vec![],
            redelegations: BTreeSet::<usize>::new(),
            did_parser: None,
            store: None,
        }
    }

    /// Validate and set [DidParser] for [ProofChain] build.
    pub async fn with_parser(mut self, mut did_parser: DidParser) -> Result<ProofChain<'a, S>> {
        self.ucan.validate(&mut did_parser).await?;
        self.did_parser = Some(Arc::new(did_parser));
        Ok(self)
    }

    async fn with_parser_arc(
        mut self,
        mut did_parser: Arc<DidParser>,
    ) -> Result<ProofChain<'a, S>> {
        self.ucan.validate(Arc::make_mut(&mut did_parser)).await?;
        self.did_parser = Some(did_parser);
        Ok(self)
    }

    /// Set [UcanJwtStore] for [ProofChain] build.
    pub fn with_store(mut self, store: &'a S) -> Self {
        self.store = Some(store);
        self
    }

    /// Instantiate a bare [ProofChain] from a [Cid], given a [UcanJwtStore].
    /// The [Cid] must resolve to a JWT token string and be available in the store.
    pub async fn from_cid(cid: &Cid, store: &'a S) -> Result<ProofChain<'a, S>> {
        let ucan_token = store.require_token(cid).await?;
        Ok(Self {
            ucan: Ucan::try_from(ucan_token)?,
            proofs: vec![],
            redelegations: BTreeSet::<usize>::new(),
            did_parser: None,
            store: Some(store),
        })
    }

    /// Get the [ProofChain]'s ucan.
    pub fn ucan(&self) -> &Ucan {
        &self.ucan
    }

    /// Get the [ProofChain]'s proofs.
    pub fn proofs(&self) -> &Vec<Self> {
        &self.proofs
    }

    /// Build-out entire [ProofChain] after setting a [UcanJwtStore] and [DidParser].
    pub fn build(mut self) -> BuildResult<'a, S> {
        let proof_chain = async move {
            let did_parser = self
                .did_parser
                .take()
                .context("No parser set for ProofChain")?;

            for cid_string in self.ucan.proofs().iter() {
                let cid = Cid::try_from(cid_string.as_str())?;

                let proof_chain =
                    ProofChain::from_cid(&cid, self.store.context("No store set for ProofChain")?)
                        .await?
                        .with_parser_arc(Arc::clone(&did_parser))
                        .await?
                        .build()
                        .await?;

                proof_chain.validate_link_to(&self.ucan)?;
                self.proofs.push(proof_chain);

                for capability in CapabilityIterator::new(&self.ucan, &PROOF_DELEGATION_SEMANTICS) {
                    match capability.with() {
                        With::Resource {
                            kind: Resource::Scoped(ProofSelection::All),
                        } => {
                            for index in 0..self.proofs.len() {
                                self.redelegations.insert(index);
                            }
                        }
                        With::Resource {
                            kind: Resource::Scoped(ProofSelection::Index(index)),
                        } => {
                            if *index < self.proofs.len() {
                                self.redelegations.insert(*index);
                            } else {
                                return Err(anyhow!(
                                    "Unable to redelegate proof; no proof at zero-based index {}",
                                    index
                                ));
                            }
                        }
                        _ => continue,
                    }
                }
            }

            Ok(ProofChain {
                ucan: self.ucan,
                proofs: self.proofs,
                redelegations: self.redelegations,
                did_parser: Some(did_parser),
                store: self.store,
            })
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            proof_chain.boxed()
        }

        #[cfg(target_arch = "wasm32")]
        {
            proof_chain.boxed_local()
        }
    }

    pub fn reduce_capabilities<Semantics, Sc, A>(
        &self,
        semantics: &Semantics,
    ) -> Vec<CapabilityInfo<Sc, A>>
    where
        Semantics: CapabilitySemantics<Sc, A>,
        Sc: Scope,
        A: Action,
    {
        // Get the set of inherited attenuations (excluding redelegations)
        // before further attenuating by own lifetime and capabilities:
        let ancestral_capability_infos: Vec<CapabilityInfo<Sc, A>> = self
            .proofs
            .iter()
            .enumerate()
            .flat_map(|(index, ancestor_chain)| {
                if self.redelegations.contains(&index) {
                    Vec::new()
                } else {
                    ancestor_chain.reduce_capabilities(semantics)
                }
            })
            .collect();

        // Get the set of capabilities that are blanket redelegated from
        // ancestor proofs (via the prf: resource):
        let mut redelegated_capability_infos: Vec<CapabilityInfo<Sc, A>> = self
            .redelegations
            .iter()
            .flat_map(|index| {
                self.proofs
                    .get(*index)
                    .unwrap()
                    .reduce_capabilities(semantics)
                    .into_iter()
                    .map(|mut info| {
                        // Redelegated capabilities should be attenuated by
                        // this UCAN's lifetime
                        info.not_before = *self.ucan.not_before();
                        info.expires_at = *self.ucan.expires_at();
                        info
                    })
            })
            .collect();

        let self_capabilities_iter = CapabilityIterator::new(&self.ucan, semantics);

        // Get the claimed attenuations of this ucan, cross-checking ancestral
        // attenuations to discover the originating authority
        let mut self_capability_infos: Vec<CapabilityInfo<Sc, A>> = if self.proofs.is_empty() {
            self_capabilities_iter
                .map(|capability| CapabilityInfo {
                    originators: BTreeSet::from_iter(vec![self.ucan.issuer().to_string()]),
                    capability,
                    not_before: *self.ucan.not_before(),
                    expires_at: *self.ucan.expires_at(),
                })
                .collect()
        } else {
            self_capabilities_iter
                .map(|capability| {
                    let mut originators = BTreeSet::<String>::new();

                    for ancestral_capability_info in ancestral_capability_infos.iter() {
                        match ancestral_capability_info.capability.enables(&capability) {
                            true => {
                                originators.extend(ancestral_capability_info.originators.clone())
                            }
                            // true => return Some(capability),
                            false => continue,
                        }
                    }

                    // If there are no related ancestral capability, then this
                    // link in the chain is considered the first originator
                    if originators.is_empty() {
                        originators.insert(self.ucan.issuer().to_string());
                    }

                    CapabilityInfo {
                        capability,
                        originators,
                        not_before: *self.ucan.not_before(),
                        expires_at: *self.ucan.expires_at(),
                    }
                })
                .collect()
        };

        self_capability_infos.append(&mut redelegated_capability_infos);

        let mut merged_capability_infos = Vec::<CapabilityInfo<Sc, A>>::new();

        // Merge redundant capabilities (accounting for redelegation), ensuring
        // that discrete originators are aggregated as we go
        'merge: while let Some(capability_info) = self_capability_infos.pop() {
            for remaining_capability_info in &mut self_capability_infos {
                if remaining_capability_info
                    .capability
                    .enables(&capability_info.capability)
                {
                    remaining_capability_info
                        .originators
                        .extend(capability_info.originators);
                    continue 'merge;
                }
            }

            merged_capability_infos.push(capability_info);
        }

        merged_capability_infos
    }

    fn validate_link_to(&self, ucan: &Ucan) -> Result<()> {
        let audience = self.ucan.audience();
        let issuer = ucan.issuer();

        match audience == issuer {
            true if self.ucan.lifetime_encompasses(ucan) => Ok(()),
            true => Err(anyhow!("Invalid UCAN link: lifetime exceeds attenuation")),
            false => Err(anyhow!(
                "Invalid UCAN link: audience {} does not match issuer {}",
                audience,
                issuer
            )),
        }
    }
}

/// Instantiate a [ProofChain] from a JWT token string reference.
impl<'a, S> TryFrom<&'a str> for ProofChain<'a, S>
where
    S: UcanJwtStore,
{
    type Error = anyhow::Error;

    fn try_from(ucan_token: &str) -> Result<Self, Self::Error> {
        ProofChain::from_str(ucan_token)
    }
}

/// Instantiate a [ProofChain] from a JWT token owned string.
impl<S> TryFrom<String> for ProofChain<'_, S>
where
    S: UcanJwtStore,
{
    type Error = anyhow::Error;

    fn try_from(ucan_token: String) -> Result<Self, Self::Error> {
        ProofChain::from_str(ucan_token.as_str())
    }
}

/// Instantiate a [ProofChain] from a JWT token owned string reference.
impl<S> FromStr for ProofChain<'_, S>
where
    S: UcanJwtStore,
{
    type Err = anyhow::Error;

    fn from_str(ucan_token: &str) -> Result<Self, Self::Err> {
        let ucan = Ucan::try_from(ucan_token)?;
        Ok(Self::from_ucan(ucan))
    }
}
