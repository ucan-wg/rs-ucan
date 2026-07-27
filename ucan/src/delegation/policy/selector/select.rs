//! Typesafe selection on [`Ipld`] values.

use super::{error::SelectorErrorReason, filter::Filter, Selectable, Selector, SelectorError};
use alloc::{string::ToString, vec, vec::Vec};
use core::{cmp::Ordering, fmt, marker::PhantomData, str::FromStr};

/// Resolve Python-style slice indices into a `(start, end)` pair clamped to `0..len`.
fn resolve_slice_indices(start: Option<i32>, end: Option<i32>, len: usize) -> (usize, usize) {
    let resolve = |idx: i32, len: usize| -> usize {
        if idx >= 0 {
            (idx.unsigned_abs() as usize).min(len)
        } else {
            len.saturating_sub(idx.unsigned_abs() as usize)
        }
    };
    let s = start.map_or(0, |i| resolve(i, len));
    let e = end.map_or(len, |i| resolve(i, len));
    (s, e.max(s))
}
use ipld_core::ipld::Ipld;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[cfg(any(test, feature = "test_utils"))]
use arbitrary::{self, Arbitrary, Unstructured};

/// Typesafe selection via [`Ipld`] selectors.
#[derive(Clone)]
pub struct Select<T> {
    filters: Vec<Filter>,
    _marker: PhantomData<T>,
}

impl<T> Serialize for Select<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Selector(self.filters.clone()).serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Select<T>
where
    Ipld: From<T>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let selector = Selector::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        Ok(Select {
            filters: selector.0,
            _marker: PhantomData,
        })
    }
}

impl<T> fmt::Debug for Select<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Select({:?})", self.filters)
    }
}

impl<T> PartialEq for Select<T> {
    fn eq(&self, other: &Self) -> bool {
        Selector(self.filters.clone()) == Selector(other.filters.clone())
    }
}

impl<T> Select<T> {
    /// Creates a new `Select` instance with the given filters.
    #[must_use]
    pub const fn new(filters: Vec<Filter>) -> Self {
        Self {
            filters,
            _marker: PhantomData,
        }
    }

    /// Checks if two selectors are related.
    #[must_use]
    pub fn is_related<U: Clone>(&self, other: &Select<U>) -> bool
    where
        Ipld: From<T> + From<U>,
    {
        Selector(self.filters.clone()).is_related(&Selector(other.filters.clone()))
    }
}

impl<T: Selectable> Select<T> {
    /// Tries to retrieve the value from the given [`Ipld`] using the selector.
    ///
    /// # Errors
    ///
    /// Returns a [`SelectorError`] if the data shape does not conform to the requested path.
    #[allow(clippy::too_many_lines)]
    pub fn get(&self, ctx: &Ipld) -> Result<T, SelectorError> {
        let got = self.filters.iter().try_fold(
            (ctx.clone(), vec![], false),
            |(ipld, mut seen_ops, is_try), op| {
                seen_ops.push(op);

                match op {
                    Filter::Try(inner) => {
                        let op: Filter = *inner.clone();
                        let ipld: Ipld =
                            Select::<Ipld>::new(vec![op]).get(ctx).unwrap_or(Ipld::Null);
                        Ok((ipld, seen_ops.clone(), true))
                    }
                    Filter::ArrayIndex(i) => {
                        let result = {
                            match ipld {
                                Ipld::List(xs) => {
                                    if xs.len() > (i32::MAX as usize) {
                                        return Err((
                                            is_try,
                                            SelectorError::from_refs(
                                                &seen_ops,
                                                SelectorErrorReason::IndexOutOfBounds,
                                            ),
                                        ));
                                    }

                                    if i.unsigned_abs() as usize > xs.len() {
                                        return Err((
                                            is_try,
                                            SelectorError::from_refs(
                                                &seen_ops,
                                                SelectorErrorReason::IndexOutOfBounds,
                                            ),
                                        ));
                                    }

                                    let idx: usize = if *i >= 0 {
                                        i.unsigned_abs() as usize
                                    } else {
                                        xs.len() - i.unsigned_abs() as usize
                                    };

                                    xs.get(idx)
                                        .ok_or((
                                            is_try,
                                            SelectorError::from_refs(
                                                &seen_ops,
                                                SelectorErrorReason::IndexOutOfBounds,
                                            ),
                                        ))
                                        .cloned()
                                }
                                Ipld::Bytes(bs) => {
                                    if bs.len() > (i32::MAX as usize) {
                                        return Err((
                                            is_try,
                                            SelectorError::from_refs(
                                                &seen_ops,
                                                SelectorErrorReason::IndexOutOfBounds,
                                            ),
                                        ));
                                    }

                                    if i.unsigned_abs() as usize > bs.len() {
                                        return Err((
                                            is_try,
                                            SelectorError::from_refs(
                                                &seen_ops,
                                                SelectorErrorReason::IndexOutOfBounds,
                                            ),
                                        ));
                                    }

                                    let idx: usize = if *i >= 0 {
                                        i.unsigned_abs() as usize
                                    } else {
                                        bs.len() - i.unsigned_abs() as usize
                                    };

                                    bs.get(idx).map(|b| Ipld::Integer(i128::from(*b))).ok_or((
                                        is_try,
                                        SelectorError::from_refs(
                                            &seen_ops,
                                            SelectorErrorReason::IndexOutOfBounds,
                                        ),
                                    ))
                                }
                                Ipld::Null
                                | Ipld::Bool(_)
                                | Ipld::Integer(_)
                                | Ipld::Float(_)
                                | Ipld::String(_)
                                | Ipld::Map(_)
                                | Ipld::Link(_) => Err((
                                    is_try,
                                    SelectorError::from_refs(
                                        &seen_ops,
                                        SelectorErrorReason::NotAList,
                                    ),
                                )),
                            }
                        };

                        Ok((result?, seen_ops.clone(), is_try))
                    }
                    Filter::Field(k) => {
                        let result = match ipld {
                            Ipld::Map(xs) => xs
                                .get(k)
                                .ok_or((
                                    is_try,
                                    SelectorError::from_refs(
                                        &seen_ops,
                                        SelectorErrorReason::KeyNotFound,
                                    ),
                                ))
                                .cloned(),
                            Ipld::Null
                            | Ipld::Bool(_)
                            | Ipld::Integer(_)
                            | Ipld::Float(_)
                            | Ipld::String(_)
                            | Ipld::Bytes(_)
                            | Ipld::List(_)
                            | Ipld::Link(_) => Err((
                                is_try,
                                SelectorError::from_refs(&seen_ops, SelectorErrorReason::NotAMap),
                            )),
                        };

                        Ok((result?, seen_ops.clone(), is_try))
                    }
                    Filter::Slice { start, end } => {
                        let result = match ipld {
                            Ipld::List(xs) => {
                                let (s, e) = resolve_slice_indices(*start, *end, xs.len());
                                Ok(Ipld::List(xs.get(s..e).unwrap_or_default().to_vec()))
                            }
                            Ipld::Bytes(bs) => {
                                let (s, e) = resolve_slice_indices(*start, *end, bs.len());
                                Ok(Ipld::Bytes(bs.get(s..e).unwrap_or_default().to_vec()))
                            }
                            Ipld::Null
                            | Ipld::Bool(_)
                            | Ipld::Integer(_)
                            | Ipld::Float(_)
                            | Ipld::String(_)
                            | Ipld::Map(_)
                            | Ipld::Link(_) => Err((
                                is_try,
                                SelectorError::from_refs(&seen_ops, SelectorErrorReason::NotAList),
                            )),
                        };

                        Ok((result?, seen_ops.clone(), is_try))
                    }
                    Filter::Values => {
                        let result = match ipld {
                            Ipld::List(xs) => Ok(Ipld::List(xs)),
                            Ipld::Map(xs) => Ok(Ipld::List(xs.values().cloned().collect())),
                            Ipld::Null
                            | Ipld::Bool(_)
                            | Ipld::Integer(_)
                            | Ipld::Float(_)
                            | Ipld::String(_)
                            | Ipld::Bytes(_)
                            | Ipld::Link(_) => Err((
                                is_try,
                                SelectorError::from_refs(
                                    &seen_ops,
                                    SelectorErrorReason::NotACollection,
                                ),
                            )),
                        };

                        Ok((result?, seen_ops.clone(), is_try))
                    }
                }
            },
        );

        let (ipld, path) = match got {
            Ok((ipld, seen_ops, _)) => Ok((ipld, seen_ops)),
            Err((is_try, ref e @ SelectorError { ref selector, .. })) => {
                if is_try {
                    Ok((Ipld::Null, selector.0.iter().collect::<Vec<_>>()))
                } else {
                    Err(e.clone())
                }
            }
        }?;

        T::try_select(ipld).map_err(|e| SelectorError::from_refs(&path, e))
    }
}

impl<T> From<Select<T>> for Ipld
where
    Ipld: From<T>,
{
    fn from(s: Select<T>) -> Self {
        Selector(s.filters).to_string().into()
    }
}

impl<T> FromStr for Select<T> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let selector = Selector::from_str(s).map_err(ParseError)?;
        Ok(Select {
            filters: selector.0,
            _marker: PhantomData,
        })
    }
}

/// Error type for parsing a selector.
#[derive(Debug, PartialEq, Error)]
#[error("Failed to parse selector: {0}")]
pub struct ParseError(#[from] nom::Err<super::error::ParseError>);

impl<T> PartialOrd for Select<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Selector(self.filters.clone()).partial_cmp(&Selector(other.filters.clone()))
    }
}

#[cfg(any(test, feature = "test_utils"))]
impl<'a, T> Arbitrary<'a> for Select<T> {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, arbitrary::Error> {
        u.arbitrary::<Vec<Filter>>()
            .map(|filters| Select::new(filters))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipld::InternalIpld;

    #[allow(clippy::expect_used)]
    mod get {
        use super::*;
        use crate::ipld::eq_with_float_nans_and_infinities;
        use proptest::prelude::*;
        use proptest_arbitrary_interop::arb;

        proptest! {
            #![proptest_config(ProptestConfig { cases: 32, ..ProptestConfig::default() })]
            #[test_log::test]
            fn test_identity(data in arb::<InternalIpld>()) {
                let selector = Select::<InternalIpld>::from_str(".")?;
                prop_assert!(eq_with_float_nans_and_infinities(&selector.get(&data.clone().into())?, &data));
            }
        }

        proptest! {
            #![proptest_config(ProptestConfig { cases: 32, ..ProptestConfig::default() })]
            #[test_log::test]
            fn test_try_missing_is_null(data in arb::<InternalIpld>()) {
                let selector = Select::<Ipld>::from_str(".foo?")?;

                let mut cleaned_data = Ipld::from(data);
                if let Ipld::Map(ref mut m) = cleaned_data {
                    m.remove("foo");
                } else if let Ipld::List(_) = cleaned_data {
                    cleaned_data = Ipld::Null;
                }

                prop_assert_eq!(selector.get(&cleaned_data)?, Ipld::Null);
            }
        }

        #[test_log::test]
        fn test_slice_list() {
            let data = Ipld::List(vec![
                Ipld::Integer(10),
                Ipld::Integer(20),
                Ipld::Integer(30),
                Ipld::Integer(40),
            ]);
            let selector = Select::<Ipld>::from_str(".[1:3]").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(
                result,
                Ipld::List(vec![Ipld::Integer(20), Ipld::Integer(30)])
            );
        }

        #[test_log::test]
        fn test_slice_list_open_end() {
            let data = Ipld::List(vec![
                Ipld::Integer(10),
                Ipld::Integer(20),
                Ipld::Integer(30),
            ]);
            let selector = Select::<Ipld>::from_str(".[1:]").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(
                result,
                Ipld::List(vec![Ipld::Integer(20), Ipld::Integer(30)])
            );
        }

        #[test_log::test]
        fn test_slice_list_open_start() {
            let data = Ipld::List(vec![
                Ipld::Integer(10),
                Ipld::Integer(20),
                Ipld::Integer(30),
            ]);
            let selector = Select::<Ipld>::from_str(".[:2]").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(
                result,
                Ipld::List(vec![Ipld::Integer(10), Ipld::Integer(20)])
            );
        }

        #[test_log::test]
        fn test_slice_negative_end() {
            let data = Ipld::List(vec![
                Ipld::Integer(10),
                Ipld::Integer(20),
                Ipld::Integer(30),
            ]);
            let selector = Select::<Ipld>::from_str(".[0:-1]").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(
                result,
                Ipld::List(vec![Ipld::Integer(10), Ipld::Integer(20)])
            );
        }

        #[test_log::test]
        fn test_byte_index() {
            let data = Ipld::Bytes(vec![0xd6, 0xa9, 0xc1, 0x8c, 0xf8, 0xc4]);
            let selector = Select::<Ipld>::from_str(".[3]").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(result, Ipld::Integer(0x8c));
        }

        #[test_log::test]
        fn test_byte_slice() {
            let data = Ipld::Bytes(vec![0xd6, 0xa9, 0xc1, 0x8c, 0xf8, 0xc4]);
            let selector = Select::<Ipld>::from_str(".[1:3]").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(result, Ipld::Bytes(vec![0xa9, 0xc1]));
        }

        #[test_log::test]
        fn test_slice_both_negative() {
            let data = Ipld::List(vec![
                Ipld::Integer(10),
                Ipld::Integer(20),
                Ipld::Integer(30),
                Ipld::Integer(40),
                Ipld::Integer(50),
            ]);
            let selector = Select::<Ipld>::from_str(".[-3:-1]").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(
                result,
                Ipld::List(vec![Ipld::Integer(30), Ipld::Integer(40)])
            );
        }

        #[test_log::test]
        fn test_slice_negative_start_open_end() {
            let data = Ipld::List(vec![
                Ipld::Integer(10),
                Ipld::Integer(20),
                Ipld::Integer(30),
            ]);
            let selector = Select::<Ipld>::from_str(".[-2:]").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(
                result,
                Ipld::List(vec![Ipld::Integer(20), Ipld::Integer(30)])
            );
        }

        #[test_log::test]
        fn test_slice_full_copy() {
            let data = Ipld::List(vec![Ipld::Integer(10), Ipld::Integer(20)]);
            let selector = Select::<Ipld>::from_str(".[:]").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(result, data);
        }

        #[test_log::test]
        fn test_slice_empty_when_start_ge_end() {
            let data = Ipld::List(vec![
                Ipld::Integer(10),
                Ipld::Integer(20),
                Ipld::Integer(30),
            ]);
            let selector = Select::<Ipld>::from_str(".[2:1]").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(result, Ipld::List(vec![]));
        }

        #[test_log::test]
        fn test_slice_out_of_bounds_clamps() {
            let data = Ipld::List(vec![Ipld::Integer(10), Ipld::Integer(20)]);
            let selector = Select::<Ipld>::from_str(".[0:100]").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(result, data);
        }

        #[test_log::test]
        fn test_byte_negative_index() {
            let data = Ipld::Bytes(vec![0xAA, 0xBB, 0xCC]);
            let selector = Select::<Ipld>::from_str(".[-1]").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(result, Ipld::Integer(0xCC));
        }

        #[test_log::test]
        fn test_byte_slice_negative() {
            let data = Ipld::Bytes(vec![0xAA, 0xBB, 0xCC, 0xDD]);
            let selector = Select::<Ipld>::from_str(".[-2:]").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(result, Ipld::Bytes(vec![0xCC, 0xDD]));
        }

        #[test_log::test]
        fn test_byte_index_out_of_bounds_with_try() {
            let data = Ipld::Bytes(vec![0xAA, 0xBB]);
            let selector = Select::<Ipld>::from_str(".[99]?").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(result, Ipld::Null);
        }

        #[test_log::test]
        fn test_slice_on_non_list_fails() {
            let data = Ipld::Integer(42);
            let selector = Select::<Ipld>::from_str(".[0:2]").expect("parse");
            assert!(selector.get(&data).is_err());
        }

        #[test_log::test]
        fn test_slice_on_non_list_with_try_returns_null() {
            let data = Ipld::Integer(42);
            let selector = Select::<Ipld>::from_str(".[0:2]?").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(result, Ipld::Null);
        }

        #[test_log::test]
        fn test_byte_index_spec_example() {
            // From the spec: bytes 0xd6a9c18cf8c4, selector .[3] => 0x8c = 140
            let data = Ipld::Bytes(vec![0xd6, 0xa9, 0xc1, 0x8c, 0xf8, 0xc4]);
            let selector = Select::<Ipld>::from_str(".[3]").expect("parse");
            let result = selector.get(&data).expect("get");
            assert_eq!(result, Ipld::Integer(140));
        }

        proptest! {
            #![proptest_config(ProptestConfig { cases: 32, ..ProptestConfig::default() })]
            #[test_log::test]
            fn test_try_missing_plus_trailing_is_null(data in arb::<InternalIpld>(), more in arb::<Vec<Filter>>()) {
                let mut filters = vec![Filter::Try(Box::new(Filter::Field("foo".into())))];

                for f in &more {
                    match f {
                        Filter::Try(_) | Filter::Values | Filter::Slice { .. } => {}
                        other @ (Filter::ArrayIndex(_) | Filter::Field(_)) => filters.push(other.clone()),
                    }
                }

                let selector: Select<Ipld> = Select::new(filters);

                let mut cleaned_data = Ipld::from(data);
                if let Ipld::Map(ref mut m) = cleaned_data {
                    m.remove("foo");
                }

                prop_assert_eq!(selector.get(&cleaned_data)?, Ipld::Null);
            }
        }
    }
}
