pub mod payload;
pub mod resolvable;

use crate::signature;

pub type Invocation<B> = signature::Envelope<payload::Payload<B>>;
