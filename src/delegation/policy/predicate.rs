use super::selector::filter::Filter;
use super::selector::{Select, SelectorError};
use crate::ipld;
use enum_as_inner::EnumAsInner;
use libipld_core::ipld::Ipld;
use multihash::Hasher;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

// FIXME Normal form?
// FIXME exract domain gen selectors first?
// FIXME rename constraint or validation or expression or something?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Predicate {
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
    Equal,            // e.g. x > 10 vs x > 10
    Conflict,         // e.g. x == 1 vs x == 2
    LhsWeaker,        // e.g. x > 10 vs x > 100 (AKA compatible but rhs narrower than lhs)
    LhsStronger,      // e.g. x > 10 vs x > 1 (AKA compatible lhs narrower than rhs)
    StrongerTogether, // e.g. x > 10 vs x < 100 (AKA both narrow each other)
    IncomparablePath, // e.g. .foo and .bar
}

impl Harmonization {
    pub fn complement(self) -> Self {
        match self {
            Harmonization::Equal => Harmonization::Conflict,
            Harmonization::Conflict => Harmonization::Equal, // FIXME Correct?
            Harmonization::LhsWeaker => Harmonization::LhsStronger,
            Harmonization::LhsStronger => Harmonization::LhsWeaker,
            Harmonization::StrongerTogether => Harmonization::StrongerTogether,
            Harmonization::IncomparablePath => Harmonization::IncomparablePath,
        }
    }

    pub fn flip(self) -> Self {
        match self {
            Harmonization::Equal => Harmonization::Equal,
            Harmonization::Conflict => Harmonization::Conflict,
            Harmonization::LhsWeaker => Harmonization::LhsStronger,
            Harmonization::LhsStronger => Harmonization::LhsWeaker,
            Harmonization::StrongerTogether => Harmonization::StrongerTogether,
            Harmonization::IncomparablePath => Harmonization::IncomparablePath,
        }
    }
}

impl Predicate {
    pub fn run(self, data: &Ipld) -> Result<bool, SelectorError> {
        Ok(match self {
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
        match (self, other) {
            (
                Predicate::Equal(lhs_selector, lhs_ipld),
                Predicate::Equal(rhs_selector, rhs_ipld),
            ) => {
                // FIXME include ctx in path?
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_ipld == rhs_ipld {
                        Harmonization::Equal
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::Equal(lhs_selector, lhs_ipld),
                Predicate::GreaterThan(rhs_selector, rhs_num),
            ) => {
                // FIXME lhs + rhs selector must be exact
                if lhs_selector.is_related(rhs_selector) {
                    if let Ok(lhs_num) = ipld::Number::try_from(lhs_ipld.0.clone()) {
                        if lhs_num > *rhs_num {
                            Harmonization::LhsStronger
                        } else {
                            Harmonization::Conflict
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::Equal(lhs_selector, lhs_ipld),
                Predicate::GreaterThanOrEqual(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if let Ok(lhs_num) = ipld::Number::try_from(lhs_ipld.0.clone()) {
                        if lhs_num >= *rhs_num {
                            Harmonization::LhsStronger
                        } else {
                            Harmonization::Conflict
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::Equal(lhs_selector, lhs_ipld),
                Predicate::LessThan(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if let Ok(lhs_num) = ipld::Number::try_from(lhs_ipld.0.clone()) {
                        if lhs_num < *rhs_num {
                            Harmonization::LhsStronger
                        } else {
                            Harmonization::Conflict
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::Equal(lhs_selector, lhs_ipld),
                Predicate::LessThanOrEqual(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if let Ok(lhs_num) = ipld::Number::try_from(lhs_ipld.0.clone()) {
                        if lhs_num <= *rhs_num {
                            Harmonization::LhsStronger
                        } else {
                            Harmonization::Conflict
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            /***********
             * Strings *
             ***********/
            (Predicate::Like(lhs_selector, lhs_str), Predicate::Like(rhs_selector, rhs_str)) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_str == rhs_str {
                            Harmonization::Equal
                        } else {
                            // FIXME actually not accurate; need to walk both in case of inner patterns
                            match (glob(lhs_str, rhs_str), glob(rhs_str, lhs_str)) {
                                (true, true) => Harmonization::StrongerTogether,
                                _ => Harmonization::Conflict,
                            }
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (Predicate::Equal(lhs_selector, lhs_ipld), Predicate::Like(rhs_selector, rhs_str)) => {
                if lhs_selector.is_related(rhs_selector) {
                    if let Ipld::String(lhs_str) = &lhs_ipld.0 {
                        if glob(&lhs_str, rhs_str) {
                            // FIXME?
                            Harmonization::LhsStronger
                        } else {
                            Harmonization::Conflict
                        }
                    } else {
                        // NOTE Predicate::Like forces this to unify as a string, so anything else fails
                        // ...so this is not *not* a type checker
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (lhs @ Predicate::Like(_, _), rhs @ Predicate::Equal(_, _)) => {
                rhs.harmonize(lhs, rhs_ctx, lhs_ctx).complement()
            }

            /****************
             * Greater Than *
             ***************/
            (
                Predicate::GreaterThan(lhs_selector, lhs_num),
                Predicate::GreaterThan(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num == rhs_num {
                            Harmonization::Equal
                        } else if lhs_num > rhs_num {
                            Harmonization::LhsStronger
                        } else {
                            Harmonization::LhsWeaker
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::GreaterThan(lhs_selector, lhs_num),
                Predicate::GreaterThanOrEqual(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num < rhs_num {
                            Harmonization::LhsWeaker
                        } else {
                            Harmonization::LhsStronger
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::GreaterThan(lhs_selector, lhs_num),
                Predicate::LessThan(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num > rhs_num {
                            Harmonization::StrongerTogether
                        } else {
                            Harmonization::Conflict
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::GreaterThan(lhs_selector, lhs_num),
                Predicate::LessThanOrEqual(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num > rhs_num {
                            Harmonization::StrongerTogether
                        } else {
                            Harmonization::Conflict
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }

            /*************************
             * Greater Than Or Equal *
             *************************/
            (
                Predicate::GreaterThanOrEqual(lhs_selector, lhs_num),
                Predicate::GreaterThanOrEqual(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num == rhs_num {
                            Harmonization::Equal
                        } else if lhs_num > rhs_num {
                            Harmonization::LhsStronger
                        } else {
                            Harmonization::LhsWeaker
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::GreaterThanOrEqual(lhs_selector, lhs_num),
                Predicate::GreaterThan(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num < rhs_num {
                            Harmonization::LhsStronger
                        } else {
                            Harmonization::LhsWeaker
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::GreaterThanOrEqual(lhs_selector, lhs_num),
                Predicate::LessThanOrEqual(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num <= rhs_num {
                            Harmonization::StrongerTogether
                        } else {
                            Harmonization::Conflict
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::GreaterThanOrEqual(lhs_selector, lhs_num),
                Predicate::LessThan(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num < rhs_num {
                            Harmonization::StrongerTogether
                        } else {
                            Harmonization::Conflict
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }

            /**********************
             * Less Than Or Equal *
             **********************/
            (
                Predicate::LessThanOrEqual(lhs_selector, lhs_num),
                Predicate::LessThanOrEqual(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num == rhs_num {
                            Harmonization::Equal
                        } else if lhs_num < rhs_num {
                            Harmonization::LhsStronger
                        } else {
                            Harmonization::LhsWeaker
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::LessThanOrEqual(lhs_selector, lhs_num),
                Predicate::LessThan(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num == rhs_num {
                            Harmonization::LhsWeaker
                        } else if lhs_num < rhs_num {
                            Harmonization::LhsStronger
                        } else {
                            Harmonization::LhsWeaker
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::LessThanOrEqual(lhs_selector, lhs_num),
                Predicate::GreaterThan(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num > rhs_num {
                            Harmonization::StrongerTogether
                        } else {
                            Harmonization::Conflict
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::LessThanOrEqual(lhs_selector, lhs_num),
                Predicate::GreaterThanOrEqual(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num >= rhs_num {
                            Harmonization::StrongerTogether
                        } else {
                            Harmonization::Conflict
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }

            /*************
             * Less Than *
             *************/
            (
                Predicate::LessThan(lhs_selector, lhs_num),
                Predicate::LessThan(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num == rhs_num {
                            Harmonization::Equal
                        } else if lhs_num < rhs_num {
                            Harmonization::LhsStronger
                        } else {
                            Harmonization::LhsWeaker
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::LessThan(lhs_selector, lhs_num),
                Predicate::LessThanOrEqual(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num == rhs_num {
                            Harmonization::LhsStronger
                        } else if lhs_num < rhs_num {
                            Harmonization::LhsStronger
                        } else {
                            Harmonization::LhsWeaker
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::LessThan(lhs_selector, lhs_num),
                Predicate::GreaterThan(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num > rhs_num {
                            Harmonization::StrongerTogether
                        } else {
                            Harmonization::Conflict
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }
            (
                Predicate::LessThan(lhs_selector, lhs_num),
                Predicate::GreaterThanOrEqual(rhs_selector, rhs_num),
            ) => {
                if lhs_selector.is_related(rhs_selector) {
                    if lhs_selector == rhs_selector {
                        if lhs_num > rhs_num {
                            Harmonization::StrongerTogether
                        } else {
                            Harmonization::Conflict
                        }
                    } else {
                        Harmonization::Conflict
                    }
                } else {
                    Harmonization::IncomparablePath
                }
            }

            /***************
             * Connectives *
             ***************/
            (_self, Predicate::Not(rhs_inner)) => {
                self.harmonize(rhs_inner, lhs_ctx, rhs_ctx).complement()
            }
            (Predicate::Not(lhs_inner), rhs) => {
                lhs_inner.harmonize(rhs, lhs_ctx, rhs_ctx).complement()
            }
            (_self, Predicate::And(and_left, and_right)) => {
                let rhs_raw_pred1: Predicate = *and_left.clone();
                let rhs_raw_pred2: Predicate = *and_right.clone();

                match (
                    self.harmonize(&rhs_raw_pred1, lhs_ctx.clone(), rhs_ctx.clone()),
                    self.harmonize(&rhs_raw_pred2, lhs_ctx, rhs_ctx),
                ) {
                    (Harmonization::Conflict, _) => Harmonization::Conflict,
                    (_, Harmonization::Conflict) => Harmonization::Conflict,
                    (Harmonization::IncomparablePath, right) => right,
                    (left, Harmonization::IncomparablePath) => left,
                    (Harmonization::Equal, rhs) => rhs,
                    (lhs, Harmonization::Equal) => lhs,
                    (Harmonization::LhsWeaker, Harmonization::LhsWeaker) => {
                        Harmonization::LhsWeaker
                    }
                    (Harmonization::LhsStronger, Harmonization::LhsStronger) => {
                        Harmonization::LhsStronger
                    }
                    (Harmonization::LhsStronger, Harmonization::LhsWeaker) => {
                        Harmonization::StrongerTogether
                    }
                    (Harmonization::LhsWeaker, Harmonization::LhsStronger) => {
                        Harmonization::StrongerTogether
                    }
                    (Harmonization::StrongerTogether, _) => Harmonization::StrongerTogether,
                    (_, Harmonization::StrongerTogether) => Harmonization::StrongerTogether,
                }
            }
            (lhs @ Predicate::And(_, _), rhs) => lhs.harmonize(rhs, lhs_ctx, rhs_ctx).flip(),
            (_self, Predicate::Or(or_left, or_right)) => {
                let rhs_raw_pred1: Predicate = *or_left.clone();
                let rhs_raw_pred2: Predicate = *or_right.clone();

                match (
                    self.harmonize(&rhs_raw_pred1, lhs_ctx.clone(), rhs_ctx.clone()),
                    self.harmonize(&rhs_raw_pred2, lhs_ctx, rhs_ctx),
                ) {
                    (Harmonization::Conflict, Harmonization::Conflict) => Harmonization::Conflict,
                    (lhs, Harmonization::Conflict) => lhs,
                    (Harmonization::Conflict, rhs) => rhs,
                    (Harmonization::IncomparablePath, right) => right,
                    (left, Harmonization::IncomparablePath) => left,
                    (Harmonization::Equal, rhs) => rhs,
                    (lhs, Harmonization::Equal) => lhs,
                    (Harmonization::LhsWeaker, Harmonization::LhsWeaker) => {
                        Harmonization::LhsWeaker
                    }
                    (Harmonization::LhsStronger, Harmonization::LhsStronger) => {
                        Harmonization::LhsStronger
                    }
                    (_, Harmonization::LhsWeaker) => Harmonization::LhsWeaker,
                    (Harmonization::LhsWeaker, _) => Harmonization::LhsWeaker,
                    (Harmonization::LhsStronger, Harmonization::StrongerTogether) => {
                        Harmonization::LhsStronger
                    }
                    (Harmonization::StrongerTogether, Harmonization::LhsStronger) => {
                        Harmonization::LhsStronger
                    }
                    (Harmonization::StrongerTogether, Harmonization::StrongerTogether) => {
                        Harmonization::StrongerTogether
                    }
                }
            }
            (lhs @ Predicate::Or(_, _), rhs) => lhs.harmonize(rhs, lhs_ctx, rhs_ctx).flip(),
            //     /******************
            //      * Quantification *
            //      ******************/
            //     Predicate::Every(rhs_selector, rhs_inner) => {
            //         let rhs_raw_pred: Predicate = *rhs_inner.clone();
            //         // TODO FIXME exact path
            //         todo!()
            //         // match self.harmonize(&rhs_raw_pred, lhs_ctx, rhs_ctx) {
            //         //     Harmonization::LhsPassed => Harmonization::LhsPassed,
            //         //     Harmonization::LhsWeaker => Harmonization::LhsWeaker,
            //         //     Harmonization::IncomparablePath => Harmonization::IncomparablePath,
            //         //     Harmonization::Conflict => {
            //         //         Harmonization::Conflict
            //         //     }
            //         // }
            //     }
            //     Predicate::Some(rhs_selector, rhs_inner) => {
            //         let rhs_raw_pred: Predicate = *rhs_inner.clone();
            //         // TODO FIXME As long as the lhs path doens't terminate earlier, then pass
            //         todo!()
            //         // match self.harmonize(&rhs_raw_pred, lhs_ctx, rhs_ctx) {
            //         //     Harmonization::LhsPassed => Harmonization::LhsPassed,
            //         //     Harmonization::LhsWeaker => Harmonization::LhsWeaker,
            //         //     Harmonization::IncomparablePath => Harmonization::IncomparablePath,
            //         //     Harmonization::Conflict => {
            //         //         Harmonization::Conflict
            //         //     }
            //         // }
            //     }
            // },
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
