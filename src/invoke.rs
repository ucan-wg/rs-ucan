mod payload;
mod resolvable;
mod serializer;

pub use payload::Payload;
pub use resolvable::Resolvable;

use crate::signature;

pub type Invocation<B> = signature::Envelope<payload::Payload<B>>;
