use super::Selector; // FIXME cycle?
use super::{error::SelectorErrorReason, filter::Filter, Selectable, SelectorError};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Select<T> {
    Get(Vec<Filter>),
    Pure(T),
}

impl<T: Selectable + Clone> Select<T> {
    pub fn is_related<U: Clone>(&self, other: &Select<U>) -> bool
    where
        Ipld: From<T> + From<U>,
    {
        match (self, other) {
            (Select::Pure(lhs_val), Select::Pure(rhs_val)) => {
                Ipld::from(lhs_val.clone()) == Ipld::from(rhs_val.clone())
            }
            (Select::Get(lhs_path), Select::Get(rhs_path)) => {
                Selector(lhs_path.clone()).is_related(&Selector(rhs_path.clone()))
            }
            _ => false,
        }
    }
    pub fn resolve(self, ctx: &Ipld) -> Result<T, SelectorError> {
        match self {
            Select::Pure(inner) => Ok(inner),
            Select::Get(ops) => {
                ops.iter()
                    .try_fold((ctx.clone(), vec![]), |(ipld, mut seen_ops), op| {
                        seen_ops.push(op);

                        match op {
                            Filter::Try(inner) => {
                                let op: Filter = *inner.clone();
                                let ipld: Ipld = Select::Get::<Ipld>(vec![op])
                                    .resolve(ctx)
                                    .unwrap_or(Ipld::Null);

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
    }
}

impl<T> From<Select<T>> for Ipld
where
    Ipld: From<T>,
{
    fn from(s: Select<T>) -> Self {
        match s {
            Select::Get(ops) => Selector(ops).to_string().into(),
            Select::Pure(inner) => inner.into(),
        }
    }
}

impl<T: PartialEq> PartialOrd for Select<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Select::Pure(inner), Select::Pure(other_inner)) => {
                if inner == other_inner {
                    Some(Ordering::Equal)
                } else {
                    None
                }
            }
            (Select::Get(ops), Select::Get(other_ops)) => {
                Selector(ops.clone()).partial_cmp(&Selector(other_ops.clone()))
            }
            _ => None,
        }
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
