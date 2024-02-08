mod payload;
mod resolvable;
mod serializer;

pub mod promise;

pub use payload::{Payload, Unresolved};
pub use resolvable::Resolvable;

use crate::signature;

pub type Invocation<T> = signature::Envelope<payload::Payload<T>>;
