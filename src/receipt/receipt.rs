use super::payload::Payload;
use crate::signature;

pub type Receipt<T> = signature::Envelope<Payload<T>>;
