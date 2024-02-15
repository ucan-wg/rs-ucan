mod payload;
mod resolvable;

pub mod agent;
pub mod promise;
pub mod store;

pub use payload::{Payload, Promised};
pub use resolvable::Resolvable;

use crate::{ability, did, did::Did, signature};

/// The complete, signed [`invocation::Payload`][Payload].
///
/// # Promises
///
/// For a version that can include [`Promise`][promise::Promise]s,
/// wrap your `T` in [`invocation::Promised`](Promised) to get
/// `Invocation<Promised<T>>`.
pub type Invocation<T, DID: Did> = signature::Envelope<payload::Payload<T, DID>, DID>;

// FIXME rename
pub type PromisedInvocation<T: Resolvable, D> = Invocation<T::Promised, D>;

// FIXME use presnet ability, too
pub type Preset = Invocation<ability::preset::Ready, did::Preset>;
pub type PresetPromised = Invocation<ability::preset::Promised, did::Preset>;

impl<T, DID: Did> Invocation<T, DID> {
    fn map_ability(self, f: impl FnOnce(T) -> T) -> Self {
        let mut payload = self.payload;
        payload.ability = f(payload.ability);
        Self {
            payload,
            signature: self.signature,
        }
    }
}
