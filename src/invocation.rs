pub mod payload;

use crate::signature;

pub type Invocation<B> = signature::Envelope<payload::Payload<B>>;
