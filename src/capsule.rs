//! Capsule type utilities.
//!
//! Capsule types are a pattern where you associate a string to type,
//! and use the tag as a key and the payload as a value in a map.
//! This helps disambiguate types when serializing and deserializing.
//!
//! Unlike a `type` field, the fact that it's on the outside of the payload
//! is often helpful in improving serializaion and deserialization performance.
//! It also avoids needing fields on nested structures where the inner types are known.
//!
//! Some simple examples include:
//!
//! ```javascript
//! {"u32": 42}
//! {"i64": 99}
//! {"coord": {"x": 1, "y": 2}}
//! {
//!   "boundary": [
//!     {"x": 1, "y": 2}, // ─┐
//!     {"x": 3, "y": 4}, //  ├─ Untagged coords inside "boundary" capsule
//!     {"x": 5, "y": 6}, //  │
//!     {"x": 7, "y": 8}  // ─┘
//!   ]
//! }
//! ```
//!
//! UCAN uses these in payload wrappers, such as [`Delegation`][crate::delegation::Delegation].

/// The primary capsule trait
///
/// # Examples
///
/// ```rust
/// # use ucan::capsule::Capsule;
/// # use std::collections::BTreeMap;
/// #
/// # #[derive(Debug, PartialEq)]
/// struct Coord {
///   x: i32,
///   y: i32
/// }
///
/// impl Capsule for Coord {
///   const TAG: &'static str = "coordinate";
/// }
///
/// let coord = Coord { x: 1, y: 2 };
/// let capsuled = BTreeMap::from_iter([(Coord::TAG.to_string(), coord)]);
///
/// assert_eq!(capsuled.get("coordinate"), Some(&Coord { x: 1, y: 2 }));
/// ````
pub trait Capsule {
    /// The tag to use when constructing or matching on the capsule
    const TAG: &'static str;
}
