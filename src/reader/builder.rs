use super::Reader;
use crate::{ability::arguments, delegation::Delegable, proof::checkable::Checkable};
use serde::{Deserialize, Serialize};

/// A helper newtype that marks a value as being a [`Delegable::Builder`].
///
/// The is often used as:
///
/// ```rust
/// # use ucan::reader::{Reader, Builder};
/// # type Env = ();
/// # let env = ();
/// let example: Reader<Env, Builder<u64>> = Reader {
///    env: env,
///    val: Builder(42),
/// };
/// ```
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Builder<T>(pub T);

impl<Env, T> Delegable for Reader<Env, T>
where
    Reader<Env, Builder<T>>: Checkable,
{
    type Builder = Reader<Env, Builder<T>>;
}

impl<A, T: Into<arguments::Named<A>>> From<Builder<T>> for arguments::Named<A> {
    fn from(builder: Builder<T>) -> Self {
        builder.0.into()
    }
}

impl<Env, T> From<Reader<Env, T>> for Reader<Env, Builder<T>> {
    fn from(reader: Reader<Env, T>) -> Self {
        reader.map(Builder)
    }
}

impl<Env, T> From<Reader<Env, Builder<T>>> for Reader<Env, T> {
    fn from(reader: Reader<Env, Builder<T>>) -> Self {
        reader.map(|b| b.0)
    }
}
