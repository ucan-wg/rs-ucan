use super::Selector;
use super::{error::SelectorErrorReason, filter::Filter, Selectable, SelectorError};
use libipld_core::ipld::Ipld;
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[derive(Clone)]
pub struct Select<T> {
    filters: Vec<Filter>,
    _marker: std::marker::PhantomData<T>,
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
    pub fn new(filters: Vec<Filter>) -> Self {
        Self {
            filters,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn is_related<U: Clone>(&self, other: &Select<U>) -> bool
    where
        Ipld: From<T> + From<U>,
    {
        Selector(self.filters.clone()).is_related(&Selector(other.filters.clone()))
    }
}

impl<T: Selectable> Select<T> {
    pub fn get(self, ctx: &Ipld) -> Result<T, SelectorError> {
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
                                    if i.abs() as usize > xs.len() {
                                        return Err((
                                            is_try,
                                            SelectorError::from_refs(
                                                &seen_ops,
                                                SelectorErrorReason::IndexOutOfBounds,
                                            ),
                                        ));
                                    };

                                    xs.get((xs.len() as i32 + *i) as usize)
                                        .ok_or((
                                            is_try,
                                            SelectorError::from_refs(
                                                &seen_ops,
                                                SelectorErrorReason::IndexOutOfBounds,
                                            ),
                                        ))
                                        .cloned()
                                }
                                _ => Err((
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
                            _ => Err((
                                is_try,
                                SelectorError::from_refs(&seen_ops, SelectorErrorReason::NotAMap),
                            )),
                        };

                        Ok((result?, seen_ops.clone(), is_try))
                    }
                    Filter::Values => {
                        let result = match ipld {
                            Ipld::List(xs) => Ok(Ipld::List(xs)),
                            Ipld::Map(xs) => Ok(Ipld::List(xs.values().cloned().collect())),
                            _ => Err((
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
                    Ok((Ipld::Null, selector.0.iter().map(|x| x).collect::<Vec<_>>()))
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
            _marker: std::marker::PhantomData,
        })
    }
}

#[derive(Debug, PartialEq, Error)]
#[error("Failed to parse selector: {0}")]
pub struct ParseError(#[from] nom::Err<super::error::ParseError>);

impl<T> PartialOrd for Select<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Selector(self.filters.clone()).partial_cmp(&Selector(other.filters.clone()))
    }
}

#[cfg(feature = "test_utils")]
impl<T: 'static> Arbitrary for Select<T> {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        prop::collection::vec(Filter::arbitrary(), 1..10)
            .prop_map(Select::new)
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipld;
    use pretty_assertions as pretty;
    use proptest::prelude::*;
    use testresult::TestResult;

    mod get {
        use super::*;

        fn nested_data() -> Ipld {
            Ipld::Map(
                vec![
                    ("name".to_string(), Ipld::String("Alice".to_string())),
                    ("age".to_string(), Ipld::Integer(42)),
                    (
                        "friends".to_string(),
                        Ipld::List(vec![
                            Ipld::String("Bob".to_string()),
                            Ipld::String("Charlie".to_string()),
                        ]),
                    ),
                ]
                .into_iter()
                .collect(),
            )
        }

        proptest! {
            #[test_log::test]
            fn test_identity(data: ipld::Newtype) {
                let selector = Select::<ipld::Newtype>::from_str(".")?;
                prop_assert_eq!(selector.get(&data.0)?, data);
            }

            #[test_log::test]
            fn test_try_missing_is_null(data: ipld::Newtype) {
                let selector = Select::<Ipld>::from_str(".foo?")?;
                let cleaned_data = match data.0.clone() {
                    Ipld::Map(mut m) => {
                        m.remove("foo").map_or(Ipld::Null, |v| v)
                    }
                    ipld => ipld
                };
                prop_assert_eq!(selector.get(&cleaned_data)?, Ipld::Null);
            }

            #[test_log::test]
            fn test_try_missing_plus_trailing_is_null(data: ipld::Newtype, more: Vec<Filter>) {
                let mut filters = vec![Filter::Try(Box::new(Filter::Field("foo".into())))];
                filters.append(&mut more.clone());

                let selector: Select<Ipld> = Select::new(filters);

                let cleaned_data = match data.0.clone() {
                    Ipld::Map(mut m) => {
                        m.remove("foo").map_or(Ipld::Null, |v| v)
                    }
                    ipld => ipld
                };
                prop_assert_eq!(selector.get(&cleaned_data)?, Ipld::Null);
            }
        }
    }
}
