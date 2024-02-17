mod payload;
mod resolvable;

pub mod agent;
pub mod promise;
pub mod store;

pub use payload::{Payload, Promised};
pub use resolvable::Resolvable;

use crate::{
    ability, did,
    did::Did,
    signature,
    time::{TimeBoundError, Timestamp},
};
use libipld_core::{cid::Cid, ipld::Ipld};
use web_time::SystemTime;

/// The complete, signed [`invocation::Payload`][Payload].
///
/// # Promises
///
/// For a version that can include [`Promise`][promise::Promise]s,
/// wrap your `A` in [`invocation::Promised`](Promised) to get
/// `Invocation<Promised<A>>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Invocation<A, DID: did::Did>(pub signature::Envelope<payload::Payload<A, DID>, DID>);

// FIXME use presnet ability, too
pub type Preset = Invocation<ability::preset::Ready, did::preset::Verifier>;
pub type PresetPromised = Invocation<ability::preset::Promised, did::preset::Verifier>;

impl<A, DID: Did> Invocation<A, DID> {
    pub fn payload(&self) -> &Payload<A, DID> {
        &self.0.payload
    }

    pub fn signature(&self) -> &signature::Witness<DID::Signature> {
        &self.0.signature
    }

    pub fn audience(&self) -> &Option<DID> {
        &self.0.payload.audience
    }

    pub fn issuer(&self) -> &DID {
        &self.0.payload.issuer
    }

    pub fn subject(&self) -> &DID {
        &self.0.payload.subject
    }

    pub fn ability(&self) -> &A {
        &self.0.payload.ability
    }

    pub fn map_ability<F, Z>(self, f: F) -> Invocation<Z, DID>
    where
        F: FnOnce(A) -> Z,
    {
        Invocation(signature::Envelope {
            payload: self.0.payload.map_ability(f),
            signature: self.0.signature,
        })
    }

    pub fn proofs(&self) -> &Vec<Cid> {
        &self.0.payload.proofs
    }

    pub fn issued_at(&self) -> &Option<Timestamp> {
        &self.0.payload.issued_at
    }

    pub fn expiration(&self) -> &Option<Timestamp> {
        &self.0.payload.expiration
    }

    pub fn check_time(&self, now: SystemTime) -> Result<(), TimeBoundError>
    where
        A: Clone,
    {
        self.0.payload.check_time(now)
    }

    pub fn try_sign(
        signer: &DID::Signer,
        payload: Payload<A, DID>,
    ) -> Result<Invocation<A, DID>, signature::SignError>
    where
        Payload<A, DID>: Clone,
    {
        let envelope = signature::Envelope::try_sign(signer, payload)?;
        Ok(Invocation(envelope))
    }
}

impl<A, DID: Did> did::Verifiable<DID> for Invocation<A, DID> {
    fn verifier(&self) -> &DID {
        &self.0.verifier()
    }
}

impl<T, DID: Did> Invocation<T, DID> {}

impl<T, DID: Did> From<Invocation<T, DID>> for Ipld {
    fn from(invocation: Invocation<T, DID>) -> Self {
        invocation.0.into()
    }
}
