use super::Selector; // FIXME cycle?
use super::{error::SelectorErrorReason, filter::Filter, Selectable, SelectorError};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::str::FromStr;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Select<T> {
    filters: Vec<Filter>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Selectable + Clone> Select<T> {
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

    pub fn get(self, ctx: &Ipld) -> Result<T, SelectorError> {
        self.filters
            .iter()
            .try_fold((ctx.clone(), vec![]), |(ipld, mut seen_ops), op| {
                seen_ops.push(op);

                match op {
                    Filter::Try(inner) => {
                        let op: Filter = *inner.clone();
                        let ipld: Ipld =
                            Select::<Ipld>::new(vec![op]).get(ctx).unwrap_or(Ipld::Null);

                        Ok((ipld, seen_ops))
                    }
                    Filter::ArrayIndex(i) => {
                        let result = {
                            match ipld {
                                Ipld::List(xs) => {
                                    if i.abs() as usize > xs.len() {
                                        return Err(SelectorError::from_refs(
                                            &seen_ops,
                                            SelectorErrorReason::IndexOutOfBounds,
                                        ));
                                    };

                                    xs.get((xs.len() as i32 + *i) as usize)
                                        .ok_or(SelectorError::from_refs(
                                            &seen_ops,
                                            SelectorErrorReason::IndexOutOfBounds,
                                        ))
                                        .cloned()
                                }
                                // FIXME behaviour on maps? type error
                                _ => Err(SelectorError::from_refs(
                                    &seen_ops,
                                    SelectorErrorReason::NotAList,
                                )),
                            }
                        };

                        Ok((result?, seen_ops))
                    }
                    Filter::Field(k) => {
                        let result = match ipld {
                            Ipld::Map(xs) => xs
                                .get(k)
                                .ok_or(SelectorError::from_refs(
                                    &seen_ops,
                                    SelectorErrorReason::KeyNotFound,
                                ))
                                .cloned(),
                            _ => Err(SelectorError::from_refs(
                                &seen_ops,
                                SelectorErrorReason::NotAMap,
                            )),
                        };

                        Ok((result?, seen_ops))
                    }
                    Filter::Values => {
                        let result = match ipld {
                            Ipld::List(xs) => Ok(Ipld::List(xs)),
                            Ipld::Map(xs) => Ok(Ipld::List(xs.values().cloned().collect())),
                            _ => Err(SelectorError::from_refs(
                                &seen_ops,
                                SelectorErrorReason::NotACollection,
                            )),
                        };

                        Ok((result?, seen_ops))
                    }
                }
            })
            .and_then(|(ipld, ref path)| {
                T::try_select(ipld).map_err(|e| SelectorError::from_refs(path, e))
            })
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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let selector = Selector::from_str(s).map_err(|_| ())?;
        Ok(Select {
            filters: selector.0,
            _marker: std::marker::PhantomData,
        })
    }
}

impl<T: PartialEq> PartialOrd for Select<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Selector(self.filters.clone()).partial_cmp(&Selector(other.filters.clone()))
    }
}

#[cfg(feature = "test_utils")]
impl<T: Arbitrary + 'static> Arbitrary for Select<T> {
    type Parameters = T::Parameters;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(t_params: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            T::arbitrary_with(t_params).prop_map(Select::Pure),
            // FIXME add params that make this actually correspond to data
            prop::collection::vec(Filter::arbitrary(), 1..10).prop_map(Select::Get),
        ]
        .boxed()
    }
}
