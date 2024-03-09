use super::selector::filter::Filter;
use super::selector::{Select, SelectorError};
use crate::ipld;
use enum_as_inner::EnumAsInner;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

// FIXME Normal form?
// FIXME exract domain gen selectors first?
// FIXME rename constraint or validation or expression or something?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Predicate {
    // Booleans
    True,
    False,

    // Comparison
    Equal(Select<ipld::Newtype>, ipld::Newtype),

    GreaterThan(Select<ipld::Number>, ipld::Number),
    GreaterThanOrEqual(Select<ipld::Number>, ipld::Number),

    LessThan(Select<ipld::Number>, ipld::Number),
    LessThanOrEqual(Select<ipld::Number>, ipld::Number),

    Like(Select<String>, String),

    // Connectives
    Not(Box<Predicate>),
    And(Box<Predicate>, Box<Predicate>),
    Or(Box<Predicate>, Box<Predicate>),

    // Collection iteration
    Every(Select<ipld::Collection>, Box<Predicate>), // ∀x ∈ xs
    Some(Select<ipld::Collection>, Box<Predicate>),  // ∃x ∈ xs
}

#[derive(Debug, Clone, PartialEq, EnumAsInner)]
pub enum Harmonization {
    IncompatiblePredicate, // Failed check
    IncomparablePath,      // LHS is ok
    LhsNarrowerOrEqual,    // LHS succeeded
    RhsNarrower,           // Succeeded, but RHS is narrower
}

impl Predicate {
    pub fn run(self, data: &Ipld) -> Result<bool, SelectorError> {
        Ok(match self {
            Predicate::True => true,
            Predicate::False => false,
            Predicate::Equal(lhs, rhs_data) => lhs.resolve(data)? == rhs_data,
            Predicate::GreaterThan(lhs, rhs_data) => lhs.resolve(data)? > rhs_data,
            Predicate::GreaterThanOrEqual(lhs, rhs_data) => lhs.resolve(data)? >= rhs_data,
            Predicate::LessThan(lhs, rhs_data) => lhs.resolve(data)? < rhs_data,
            Predicate::LessThanOrEqual(lhs, rhs_data) => lhs.resolve(data)? <= rhs_data,
            Predicate::Like(lhs, rhs_data) => glob(&lhs.resolve(data)?, &rhs_data),
            Predicate::Not(inner) => !inner.run(data)?,
            Predicate::And(lhs, rhs) => lhs.run(data)? && rhs.run(data)?,
            Predicate::Or(lhs, rhs) => lhs.run(data)? || rhs.run(data)?,
            Predicate::Every(xs, p) => xs
                .resolve(data)?
                .to_vec()
                .iter()
                .try_fold(true, |acc, nt| Ok(acc && p.clone().run(&nt.0)?))?,
            Predicate::Some(xs, p) => {
                let pred = p.clone();

                xs.resolve(data)?
                    .to_vec()
                    .iter()
                    .try_fold(true, |acc, nt| Ok(acc || pred.clone().run(&nt.0)?))?
            }
        })
    }

    // FIXME check paths are subsets, becase that changes some of these
    pub fn harmonize(
        &self,
        other: &Self,
        lhs_ctx: Vec<Filter>,
        rhs_ctx: Vec<Filter>,
    ) -> Harmonization {
        match self {
            Predicate::True => match other {
                Predicate::True => Harmonization::LhsNarrowerOrEqual,
                _ => todo!(),
            },
            // FIXME this should generally fail always? But how does it compare?
            Predicate::False => match other {
                // FIXME correct?
                Predicate::False => Harmonization::LhsNarrowerOrEqual,
                _ => todo!(),
            },
            Predicate::Equal(lhs_selector, lhs_ipld) => match other {
                Predicate::Equal(rhs_selector, rhs_ipld) => {
                    // FIXME include ctx in path?
                    if lhs_selector.is_related(rhs_selector) {
                        if lhs_ipld == rhs_ipld {
                            Harmonization::LhsNarrowerOrEqual
                        } else {
                            Harmonization::IncompatiblePredicate
                        }
                    } else {
                        Harmonization::IncomparablePath
                    }
                }

                /************
                 * Numerics *
                 ************/
                Predicate::GreaterThan(rhs_selector, rhs_num) => {
                    if lhs_selector.is_related(rhs_selector) {
                        if let Ok(lhs_num) = ipld::Number::try_from(lhs_ipld.0.clone()) {
                            if lhs_num > *rhs_num {
                                Harmonization::LhsNarrowerOrEqual
                            } else {
                                Harmonization::IncompatiblePredicate
                            }
                        } else {
                            Harmonization::IncompatiblePredicate
                        }
                    } else {
                        Harmonization::IncomparablePath
                    }
                }
                Predicate::GreaterThanOrEqual(rhs_selector, rhs_num) => {
                    if lhs_selector.is_related(rhs_selector) {
                        if let Ok(lhs_num) = ipld::Number::try_from(lhs_ipld.0.clone()) {
                            if lhs_num >= *rhs_num {
                                Harmonization::LhsNarrowerOrEqual
                            } else {
                                Harmonization::IncompatiblePredicate
                            }
                        } else {
                            Harmonization::IncompatiblePredicate
                        }
                    } else {
                        Harmonization::IncomparablePath
                    }
                }
                Predicate::LessThan(rhs_selector, rhs_num) => {
                    if lhs_selector.is_related(rhs_selector) {
                        if let Ok(lhs_num) = ipld::Number::try_from(lhs_ipld.0.clone()) {
                            if lhs_num < *rhs_num {
                                Harmonization::LhsNarrowerOrEqual
                            } else {
                                Harmonization::IncompatiblePredicate
                            }
                        } else {
                            Harmonization::IncompatiblePredicate
                        }
                    } else {
                        Harmonization::IncomparablePath
                    }
                }
                Predicate::LessThanOrEqual(rhs_selector, rhs_num) => {
                    if lhs_selector.is_related(rhs_selector) {
                        if let Ok(lhs_num) = ipld::Number::try_from(lhs_ipld.0.clone()) {
                            if lhs_num <= *rhs_num {
                                Harmonization::LhsNarrowerOrEqual
                            } else {
                                Harmonization::IncompatiblePredicate
                            }
                        } else {
                            Harmonization::IncompatiblePredicate
                        }
                    } else {
                        Harmonization::IncomparablePath
                    }
                }
                /**********
                 * String *
                 **********/
                Predicate::Like(rhs_selector, rhs_str) => {
                    if lhs_selector.is_related(rhs_selector) {
                        if let Ok(lhs_str) = String::try_from(*lhs_ipld) {
                            if glob(&lhs_str, rhs_str) {
                                Harmonization::LhsNarrowerOrEqual
                            } else {
                                Harmonization::IncompatiblePredicate
                            }
                        } else {
                            Harmonization::IncompatiblePredicate
                        }
                    } else {
                        Harmonization::IncomparablePath
                    }
                }

                /***************
                 * Connectives *
                 ***************/
                Predicate::Not(rhs_inner) => {
                    let rhs_raw_pred: Predicate = **rhs_inner;
                    match self.harmonize(&rhs_raw_pred, lhs_ctx, rhs_ctx) {
                        Harmonization::LhsNarrowerOrEqual => Harmonization::RhsNarrower,
                        Harmonization::RhsNarrower => Harmonization::LhsNarrowerOrEqual,
                        Harmonization::IncomparablePath => Harmonization::IncomparablePath,
                        // FIXME double check
                        Harmonization::IncompatiblePredicate => Harmonization::LhsNarrowerOrEqual,
                    }
                }
                Predicate::And(and_left, and_right) => {
                    let rhs_raw_pred1: Predicate = **and_left;
                    let rhs_raw_pred2: Predicate = **and_right;

                    match (
                        self.harmonize(&rhs_raw_pred1, lhs_ctx.clone(), rhs_ctx.clone()),
                        self.harmonize(&rhs_raw_pred2, lhs_ctx, rhs_ctx),
                    ) {
                        (Harmonization::LhsNarrowerOrEqual, Harmonization::LhsNarrowerOrEqual) => {
                            Harmonization::LhsNarrowerOrEqual
                        }
                        (Harmonization::RhsNarrower, Harmonization::RhsNarrower) => {
                            Harmonization::RhsNarrower
                        }
                        (Harmonization::LhsNarrowerOrEqual, Harmonization::RhsNarrower) => {
                            Harmonization::IncompatiblePredicate
                        }
                        (Harmonization::RhsNarrower, Harmonization::LhsNarrowerOrEqual) => {
                            Harmonization::IncompatiblePredicate
                        }
                        (Harmonization::IncomparablePath, right) => right,
                        (left, Harmonization::IncomparablePath) => left,
                        (Harmonization::IncompatiblePredicate, _) => {
                            Harmonization::IncompatiblePredicate
                        }
                        (_, Harmonization::IncompatiblePredicate) => {
                            Harmonization::IncompatiblePredicate
                        }
                    }
                }
                Predicate::Or(or_left, or_right) => {
                    let rhs_raw_pred1: Predicate = *or_left.clone();
                    let rhs_raw_pred2: Predicate = *or_right.clone();

                    match (
                        self.harmonize(&rhs_raw_pred1, lhs_ctx.clone(), rhs_ctx.clone()),
                        self.harmonize(&rhs_raw_pred2, lhs_ctx, rhs_ctx),
                    ) {
                        (Harmonization::LhsNarrowerOrEqual, Harmonization::LhsNarrowerOrEqual) => {
                            Harmonization::LhsNarrowerOrEqual
                        }
                        (Harmonization::RhsNarrower, Harmonization::RhsNarrower) => {
                            Harmonization::RhsNarrower
                        }
                        (Harmonization::LhsNarrowerOrEqual, Harmonization::RhsNarrower) => {
                            Harmonization::LhsNarrowerOrEqual
                        }
                        (Harmonization::RhsNarrower, Harmonization::LhsNarrowerOrEqual) => {
                            Harmonization::LhsNarrowerOrEqual
                        }
                        (Harmonization::IncomparablePath, right) => right,
                        (left, Harmonization::IncomparablePath) => left,
                        (Harmonization::IncompatiblePredicate, _) => {
                            Harmonization::IncompatiblePredicate
                        }
                        (_, Harmonization::IncompatiblePredicate) => {
                            Harmonization::IncompatiblePredicate
                        }
                    }
                }
                /******************
                 * Quantification *
                 ******************/
                Predicate::Every(rhs_selector, rhs_inner) => {
                    let rhs_raw_pred: Predicate = *rhs_inner.clone();
                    todo!()
                    // match self.harmonize(&rhs_raw_pred, lhs_ctx, rhs_ctx) {
                    //     Harmonization::LhsNarrowerOrEqual => Harmonization::LhsNarrowerOrEqual,
                    //     Harmonization::RhsNarrower => Harmonization::RhsNarrower,
                    //     Harmonization::IncomparablePath => Harmonization::IncomparablePath,
                    //     Harmonization::IncompatiblePredicate => {
                    //         Harmonization::IncompatiblePredicate
                    //     }
                    // }
                }
                Predicate::Some(rhs_selector, rhs_inner) => {
                    let rhs_raw_pred: Predicate = *rhs_inner.clone();
                    todo!()
                    // match self.harmonize(&rhs_raw_pred, lhs_ctx, rhs_ctx) {
                    //     Harmonization::LhsNarrowerOrEqual => Harmonization::LhsNarrowerOrEqual,
                    //     Harmonization::RhsNarrower => Harmonization::RhsNarrower,
                    //     Harmonization::IncomparablePath => Harmonization::IncomparablePath,
                    //     Harmonization::IncompatiblePredicate => {
                    //         Harmonization::IncompatiblePredicate
                    //     }
                    // }
                }
                _ => todo!(), // FIXME
            },
            _ => todo!(),
        }
    }
}

pub fn glob(input: &String, pattern: &String) -> bool {
    let mut chars = input.chars();
    let mut like = pattern.chars();

    loop {
        match (chars.next(), like.next()) {
            (Some(i), Some(p)) => {
                if p == '*' {
                    return true;
                } else if i != p {
                    return false;
                }
            }
            (Some(_), None) => {
                return false; // FIXME correct?
            }
            (None, Some(p)) => {
                if p == '*' {
                    return true;
                }
            }
            (None, None) => {
                return true;
            }
        }
    }
}

impl From<Predicate> for Ipld {
    fn from(p: Predicate) -> Self {
        match p {
            Predicate::True => Ipld::Bool(true),
            Predicate::False => Ipld::Bool(false),
            Predicate::Equal(lhs, rhs) => {
                Ipld::List(vec![Ipld::String("==".to_string()), lhs.into(), rhs.into()])
            }
            Predicate::GreaterThan(lhs, rhs) => {
                Ipld::List(vec![Ipld::String(">".to_string()), lhs.into(), rhs.into()])
            }
            Predicate::GreaterThanOrEqual(lhs, rhs) => {
                Ipld::List(vec![Ipld::String(">=".to_string()), lhs.into(), rhs.into()])
            }
            Predicate::LessThan(lhs, rhs) => {
                Ipld::List(vec![Ipld::String("<".to_string()), lhs.into(), rhs.into()])
            }
            Predicate::LessThanOrEqual(lhs, rhs) => {
                Ipld::List(vec![Ipld::String("<=".to_string()), lhs.into(), rhs.into()])
            }
            Predicate::Like(lhs, rhs) => Ipld::List(vec![
                Ipld::String("like".to_string()),
                lhs.into(),
                rhs.into(),
            ]),
            Predicate::Not(inner) => {
                let unboxed = *inner;
                Ipld::List(vec![Ipld::String("not".to_string()), unboxed.into()])
            }
            Predicate::And(lhs, rhs) => Ipld::List(vec![
                Ipld::String("and".to_string()),
                (*lhs).into(),
                (*rhs).into(),
            ]),
            Predicate::Or(lhs, rhs) => Ipld::List(vec![
                Ipld::String("or".to_string()),
                (*lhs).into(),
                (*rhs).into(),
            ]),
            Predicate::Every(xs, p) => Ipld::List(vec![
                Ipld::String("every".to_string()),
                xs.into(),
                (*p).into(),
            ]),
            Predicate::Some(xs, p) => Ipld::List(vec![
                Ipld::String("some".to_string()),
                xs.into(),
                (*p).into(),
            ]),
        }
    }
}

#[cfg(feature = "test_utils")]
impl Arbitrary for Predicate {
    type Parameters = (); // FIXME?
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_params: Self::Parameters) -> Self::Strategy {
        let leaf = prop_oneof![
            Just(Predicate::True),
            Just(Predicate::False),
            (Select::arbitrary(), Select::arbitrary())
                .prop_map(|(lhs, rhs)| { Predicate::Equal(lhs, rhs) }),
            (Select::arbitrary(), Select::arbitrary())
                .prop_map(|(lhs, rhs)| { Predicate::GreaterThan(lhs, rhs) }),
            (Select::arbitrary(), Select::arbitrary())
                .prop_map(|(lhs, rhs)| { Predicate::GreaterThanOrEqual(lhs, rhs) }),
            (Select::arbitrary(), Select::arbitrary())
                .prop_map(|(lhs, rhs)| { Predicate::LessThan(lhs, rhs) }),
            (Select::arbitrary(), Select::arbitrary())
                .prop_map(|(lhs, rhs)| { Predicate::LessThanOrEqual(lhs, rhs) }),
            (Select::arbitrary(), Select::arbitrary())
                .prop_map(|(lhs, rhs)| { Predicate::Like(lhs, rhs) })
        ];

        let connective = leaf.clone().prop_recursive(8, 16, 4, |inner| {
            prop_oneof![
                (inner.clone(), inner.clone())
                    .prop_map(|(lhs, rhs)| { Predicate::And(Box::new(lhs), Box::new(rhs)) }),
                (inner.clone(), inner.clone())
                    .prop_map(|(lhs, rhs)| { Predicate::Or(Box::new(lhs), Box::new(rhs)) }),
            ]
        });

        let quantified = leaf.clone().prop_recursive(8, 16, 4, |inner| {
            prop_oneof![
                (Select::arbitrary(), inner.clone())
                    .prop_map(|(xs, p)| { Predicate::Every(xs, Box::new(p)) }),
                (Select::arbitrary(), inner.clone())
                    .prop_map(|(xs, p)| { Predicate::Some(xs, Box::new(p)) }),
            ]
        });

        prop_oneof![leaf, connective, quantified].boxed()
    }
}
