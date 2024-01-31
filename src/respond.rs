mod payload;
mod responds;

pub use payload::Payload;
pub use responds::Responds;

use crate::signature;

pub type Receipt<T> = signature::Envelope<Payload<T>>;
