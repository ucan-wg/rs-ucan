mod payload;
mod resolvable;

pub mod promise;
pub mod store;

use crate::did;
pub use payload::{Payload, Promised};
pub use resolvable::Resolvable;

use crate::signature;

/// The complete, signed [`invocation::Payload`][Payload].
///
/// # Promises
///
/// For a version that can include [`Promise`][promise::Promise]s,
/// wrap your `T` in [`invocation::Promised`](Promised) to get
/// `Invocation<Promised<T>>`.
pub type Invocation<T, D> = signature::Envelope<payload::Payload<T, D>>;

// FIXME use presnet ability, too
pub type Preset<T> = Invocation<T, did::Preset>;
