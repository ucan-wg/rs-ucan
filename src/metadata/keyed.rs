//! Keys for metadata fields (for parsing and guarding entries)

use libipld_core::ipld::Ipld;

/// A parser trait for types that can be used as metadata fields.
///
/// # Examples
///
/// ```rust
/// # use ucan::metadata::Keyed;
/// # use std::collections::BTreeMap;
/// # use libipld::{ipld, ipld::Ipld};
/// #
/// pub struct MyKeyed {
///    pub foo: String,
///    pub bar: u32,
/// }
///
/// impl Keyed for MyKeyed {
///   const KEY: &'static str = "my/entry";
/// }
/// assert_eq!(MyKeyed::KEY, "my/entry");
///
/// let kv: BTreeMap<String, Ipld> = BTreeMap::from_iter([
///   (MyKeyed::KEY.into(), ipld!({"foo": "hello world", "bar": 42}))
/// ]);
///
/// assert_eq!(kv.get(MyKeyed::KEY), Some(&ipld!({"foo": "hello world", "bar": 42})));
/// ```
pub trait Keyed {
    /// The (string) key for this entry
    ///
    /// These should be unique per `Keyed` type to avoid parser collisions.
    /// Even if a duplicate key is used, the shape of the contained data can
    /// also be used to disambiguate.
    const KEY: &'static str;
}

// FIXME keyed?
/// A parser trait for one-or-more unioned types that can be used as metadata fields.
///
/// [`MultiKeyed`] may be composed of a single entry, or the union of sevveral e.g. via enum.
pub trait MultiKeyed: TryFrom<Ipld> + Into<Ipld> {
    /// The (string) keys for an enum merging multiple [`Keyed`]s
    const KEYS: &'static [&'static str];
}

impl<T: Keyed + TryFrom<Ipld> + Into<Ipld>> MultiKeyed for T {
    const KEYS: &'static [&'static str] = &[T::KEY];
}
