mod payload;
mod promise;
mod resolvable;
mod serializer;

pub use payload::{Payload, Unresolved};
pub use promise::Promise;
pub use resolvable::Resolvable;

use crate::signature;

pub type Invocation<B, E> = signature::Envelope<payload::Payload<B, E>>;
