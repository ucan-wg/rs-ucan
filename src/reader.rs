use crate::{
    ability::{arguments, command::ToCommand},
    delegation::Delegatable,
    invocation::Resolvable,
    proof::{checkable::Checkable, same::CheckSame},
};
use serde::{Deserialize, Serialize};

// NOTE to self: this is helpful as a common container to lift various FFI into
#[derive(Clone, PartialEq, Debug)]
pub struct Reader<Env, T> {
    pub env: Env,
    pub val: T,
}

impl<Env, T: Into<arguments::Named>> From<Reader<Env, T>> for arguments::Named {
    fn from(reader: Reader<Env, T>) -> Self {
        reader.val.into()
    }
}

// NOTE plug this into Reader<Env, T> like: Reader<Resolved<Dynamic>>
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Builder<T>(pub T);

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Promised<T>(pub T);

impl<T: Into<arguments::Named>> From<Builder<T>> for arguments::Named {
    fn from(builder: Builder<T>) -> Self {
        builder.0.into()
    }
}

impl<T: Into<arguments::Named>> From<Promised<T>> for arguments::Named {
    fn from(promised: Promised<T>) -> Self {
        promised.0.into()
    }
}

impl<Env, T> Reader<Env, T> {
    pub fn map<F, U>(self, func: F) -> Reader<Env, U>
    where
        F: FnOnce(T) -> U,
    {
        Reader {
            env: self.env,
            val: func(self.val),
        }
    }

    pub fn map_env<F, NewEnv>(self, func: F) -> Reader<NewEnv, T>
    where
        F: FnOnce(Env) -> NewEnv,
    {
        Reader {
            env: func(self.env),
            val: self.val,
        }
    }

    pub fn local<F, G, U>(&self, modify_env: F, closure: G) -> U
    where
        T: Clone,
        Env: Clone,
        F: Fn(Env) -> Env,
        G: Fn(Reader<Env, T>) -> U,
    {
        closure(Reader {
            val: self.val.clone(),
            env: modify_env(self.env.clone()),
        })
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

impl<Env, T: ToCommand + Into<arguments::Named>> Delegatable for Reader<Env, T> {
    type Builder = Reader<Env, Builder<T>>;
}

impl<Env, T: ToCommand + Into<arguments::Named>> Resolvable for Reader<Env, T> {
    type Promised = Reader<Env, Promised<T>>;
}

impl<Env: ToCommand, T> ToCommand for Reader<Env, T> {
    fn to_command(&self) -> String {
        self.env.to_command()
    }
}

impl<Env: Checkable, T> Checkable for Reader<Env, T>
where
    Reader<Env, T>: CheckSame,
{
    type Hierarchy = Env::Hierarchy;
}
