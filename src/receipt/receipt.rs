use super::payload::Payload;
use crate::signature;

pub type Receipt<T, E> = signature::Envelope<Payload<T, E>>;
