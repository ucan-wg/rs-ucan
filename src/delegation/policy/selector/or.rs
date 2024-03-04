use super::{error::SelectorErrorReason, op::SelectorOp, SelectorError};
use crate::delegation::policy::ir::TryFromIpld;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelectorOr<T> {
    Get(Vec<SelectorOp>),
    Pure(T),
}

impl<T: TryFromIpld> SelectorOr<T> {
    pub fn resolve(self, ctx: &Ipld) -> Result<T, SelectorError> {
        match self {
            SelectorOr::Pure(inner) => Ok(inner),
            SelectorOr::Get(ops) => {
                ops.iter()
                    .try_fold((ctx.clone(), vec![]), |(ipld, mut seen_ops), op| {
                        seen_ops.push(op);

                        match op {
                            SelectorOp::Try(inner) => {
                                let op: SelectorOp = *inner.clone();
                                let ipld: Ipld = SelectorOr::Get::<Ipld>(vec![op])
                                    .resolve(ctx)
                                    .unwrap_or(Ipld::Null);

                                Ok((ipld, seen_ops))
                            }
                            SelectorOp::ArrayIndex(i) => {
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
                            SelectorOp::Field(k) => {
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
                            SelectorOp::Values => {
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
                        T::try_from_ipld(ipld).map_err(|e| SelectorError::from_refs(path, e))
                    })
            }
        }
    }
}

#[cfg(feature = "test_utils")]
impl<T: Arbitrary + 'static> Arbitrary for SelectorOr<T> {
    type Parameters = T::Parameters;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(t_params: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            T::arbitrary_with(t_params).prop_map(SelectorOr::Pure),
            // FIXME add params that make this actually correspond to data
            prop::collection::vec(SelectorOp::arbitrary(), 1..10).prop_map(SelectorOr::Get),
        ]
        .boxed()
    }
}
