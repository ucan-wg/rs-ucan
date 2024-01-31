pub mod payload;
pub mod resolvable;
mod serializer;

use crate::signature;

pub type Invocation<B> = signature::Envelope<payload::Payload<B>>;
