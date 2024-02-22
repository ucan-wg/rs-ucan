use crate::ability::{arguments, command::ToCommand, parse::ParseAbilityError};
use libipld_core::ipld::Ipld;

/// A struct that attaches an ambient environment to a value.
///
/// This is a simple way to perform runtime [dependency injection][DI] in a way
/// that plumbs through traits.
///
/// This is helpful for dependency injection and/or passing around values that
/// would otherwise need to be threaded through next to the value.
///
/// This is loosely based on the [functional `Reader`][SO] type,
/// but is not implemented with forced purity. Many of the "ambient" features
/// and guarantees of the [functional `Reader`][SO] monad are not present here.
///
/// # Examples
///
/// ```rust
/// # use ucan::reader::Reader;
/// # use std::string::ToString;
/// #
/// struct Config {
///   name: String,
///   formatter: Box<dyn Fn(String) -> String>,
///   trimmer: Box<dyn Fn(String) -> String>,
/// }
///
/// fn run<T: ToString>(r: Reader<Config, T>) -> String {
///   let formatted = (r.env.formatter)(r.val.to_string());
///   (r.env.trimmer)(formatted)
/// }
///
/// let cfg1 = Config {
///     name: "cfg1".into(),
///     formatter: Box::new(|s| s.to_uppercase()),
///     trimmer: Box::new(|mut s| s.trim().into())
/// };
///
/// let cfg2 = Config {
///     name: "cfg2".into(),
///     formatter: Box::new(|s| s.to_lowercase()),
///     trimmer: Box::new(|mut s| s.split_off(5).into())
/// };
///
///
/// let reader1 = Reader {
///    env: cfg1,
///    val: " value",
/// };
///
/// let reader2 = Reader {
///    env: cfg2,
///    val: " value",
/// };
///
/// assert_eq!(run(reader1), "VALUE");
/// assert_eq!(run(reader2), "e");
/// ```
///
/// [SO]: https://stackoverflow.com/questions/14178889/what-is-the-purpose-of-the-reader-monad
/// [DI]: https://en.wikipedia.org/wiki/Dependency_injection
#[derive(Clone, PartialEq, Debug)]
pub struct Reader<Env, T> {
    /// The environment (or configuration) being passed with the value
    pub env: Env,

    /// The raw value
    pub val: T,
}

impl<Env, T> Reader<Env, T> {
    /// Map a function over the `val` of the [`Reader`]
    pub fn map<F, U>(self, func: F) -> Reader<Env, U>
    where
        F: FnOnce(T) -> U,
    {
        Reader {
            env: self.env,
            val: func(self.val),
        }
    }

    /// Modify the `env` field of the [`Reader`]
    pub fn map_env<F, NewEnv>(self, func: F) -> Reader<NewEnv, T>
    where
        F: FnOnce(Env) -> NewEnv,
    {
        Reader {
            env: func(self.env),
            val: self.val,
        }
    }

    /// Temporarily modify the environment
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ucan::reader::Reader;
    /// # use std::string::ToString;
    /// #
    /// # #[derive(Clone)]
    /// struct Config<'a> {
    ///   name: String,
    ///   formatter: &'a dyn Fn(String) -> String,
    ///   trimmer: &'a dyn Fn(String) -> String,
    /// }
    ///
    /// fn run<T: ToString>(r: Reader<Config, T>) -> String {
    ///   let formatted = (r.env.formatter)(r.val.to_string());
    ///   (r.env.trimmer)(formatted)
    /// }
    ///
    /// let cfg = Config {
    ///     name: "cfg1".into(),
    ///     formatter: &|s| s.to_uppercase(),
    ///     trimmer: &|mut s| s.trim().into()
    /// };
    ///
    /// let my_reader = Reader {
    ///    env: cfg,
    ///    val: " value",
    /// };
    ///
    /// assert_eq!(run(my_reader.clone()), "VALUE");
    ///
    /// // Modify the env locally
    /// let observed = my_reader.clone().local(|mut env| {
    ///   // Modifying env
    ///   env.trimmer = &|mut s: String| s.split_off(5).into();
    ///   env
    /// }, |r| run(r)); // Running
    /// assert_eq!(observed, "E");
    ///
    /// // Back to normal (the above was in fact "local")
    /// assert_eq!(run(my_reader.clone()), "VALUE");
    /// ```
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

impl<Env, A, T: Into<arguments::Named<A>>> From<Reader<Env, T>> for arguments::Named<A> {
    fn from(reader: Reader<Env, T>) -> Self {
        reader.val.into()
    }
}

impl<Env: ToCommand, T> ToCommand for Reader<Env, T> {
    fn to_command(&self) -> String {
        self.env.to_command()
    }
}

impl<Env: Default, T: TryFrom<arguments::Named<Ipld>>> TryFrom<arguments::Named<Ipld>>
    for Reader<Env, T>
{
    type Error = ParseAbilityError<<T as TryFrom<arguments::Named<Ipld>>>::Error>;

    fn try_from(args: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        Ok(Reader {
            env: Default::default(),
            val: T::try_from(args).map_err(ParseAbilityError::InvalidArgs)?,
        })
    }
}
