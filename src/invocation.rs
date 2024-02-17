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
/// wrap your `T` in [`invocation::Promised`](Promised) to get
/// `Invocation<Promised<T>>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Invocation<T, DID: did::Did>(pub signature::Envelope<payload::Payload<T, DID>, DID>);

// FIXME use presnet ability, too
pub type Preset = Invocation<ability::preset::Ready, did::preset::Verifier>;
pub type PresetPromised = Invocation<ability::preset::Promised, did::preset::Verifier>;

impl<T: Clone, DID: Did + Clone> Invocation<T, DID> {
    pub fn payload(&self) -> &Payload<T, DID> {
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

    pub fn ability(&self) -> &T {
        &self.0.payload.ability
    }

    pub fn proofs(&self) -> &Vec<Cid> {
        &self.0.payload.proofs
    }

    pub fn issued_at(&self) -> &Option<Timestamp> {
        &self.0.payload.issued_at
    }

    pub fn check_time(&self, now: SystemTime) -> Result<(), TimeBoundError> {
        self.0.payload.check_time(now)
    }

    pub fn try_sign(
        signer: &DID::Signer,
        payload: Payload<T, DID>,
    ) -> Result<Invocation<T, DID>, signature::SignError> {
        let envelope = signature::Envelope::try_sign(signer, payload)?;
        Ok(Invocation(envelope))
    }
}

impl<T, DID: Did> did::Verifiable<DID> for Invocation<T, DID> {
    fn verifier(&self) -> &DID {
        &self.0.verifier()
    }
}

impl<T, DID: Did> Invocation<T, DID> {
    pub fn map_ability(self, f: impl FnOnce(T) -> T) -> Self {
        let mut payload = self.0.payload;
        payload.ability = f(payload.ability);
        Invocation(signature::Envelope {
            payload,
            signature: self.0.signature,
        })
    }
}

impl<T, DID: Did> From<Invocation<T, DID>> for Ipld {
    fn from(invocation: Invocation<T, DID>) -> Self {
        invocation.0.into()
    }
}
