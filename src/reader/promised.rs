use super::Reader;
use crate::ability::{arguments, command::ToCommand};
use serde::{Deserialize, Serialize};

/// A helper newtype that marks a value as being a [`Resolvable::Promised`][crate::invocation::Resolvable::Promised].
///
/// Despite this being the intention, due to constraits, the consuming type needs to
/// implement the [`Resolvable`][crate::invocation::Resolvable] trait.
/// For example, there is a `wasm_bindgen` implementation in this crate if
/// compiled for `wasm32`.
///
/// The is often used as:
///
/// ```rust
/// # use ucan::reader::{Reader, Promised};
/// # type Env = ();
/// # let env = ();
/// let example: Reader<Env, Promised<u64>> = Reader {
///    env: env,
///    val: Promised(42),
/// };
/// ```
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Promised<T>(pub T);

impl<A, T: Into<arguments::Named<A>>> From<Promised<T>> for arguments::Named<A> {
    fn from(promised: Promised<T>) -> Self {
        promised.0.into()
    }
}
impl<Env, T> From<Reader<Env, T>> for Reader<Env, Promised<T>> {
    fn from(reader: Reader<Env, T>) -> Self {
        reader.map(Promised)
    }
}

impl<Env, T: ToCommand> From<Reader<Env, Promised<T>>> for Reader<Env, T> {
    fn from(reader: Reader<Env, Promised<T>>) -> Self {
        reader.map(|p| p.0)
    }
}
