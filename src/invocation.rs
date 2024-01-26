pub mod payload;

use crate::signature;
use payload::Payload;

pub type Invocation<B> = signature::Envelope<Payload<B>>;
