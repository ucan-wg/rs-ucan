use super::selector::filter::Filter;
use super::selector::{Select, SelectorError};
use crate::ipld;
use enum_as_inner::EnumAsInner;
use libipld_core::ipld::Ipld;
use std::str::FromStr;
use thiserror::Error;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[derive(Debug, Clone, PartialEq)]
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
    // FIXME make &self?
    pub fn run(self, data: &Ipld) -> Result<bool, SelectorError> {
        Ok(match self {
            Predicate::Equal(lhs, rhs_data) => lhs.get(data)? == rhs_data,
            Predicate::GreaterThan(lhs, rhs_data) => lhs.get(data)? > rhs_data,
            Predicate::GreaterThanOrEqual(lhs, rhs_data) => lhs.get(data)? >= rhs_data,
            Predicate::LessThan(lhs, rhs_data) => lhs.get(data)? < rhs_data,
            Predicate::LessThanOrEqual(lhs, rhs_data) => lhs.get(data)? <= rhs_data,
            Predicate::Like(lhs, rhs_data) => glob(&lhs.get(data)?, &rhs_data),
            Predicate::Not(inner) => !inner.run(data)?,
            Predicate::And(lhs, rhs) => lhs.run(data)? && rhs.run(data)?,
            Predicate::Or(lhs, rhs) => lhs.run(data)? || rhs.run(data)?,
            Predicate::Every(xs, p) => xs
                .get(data)?
                .to_vec()
                .iter()
                .try_fold(true, |acc, each_datum| {
                    Ok(acc && p.clone().run(&each_datum.0)?)
                })?,
            Predicate::Some(xs, p) => xs
                .get(data)?
                .to_vec()
                .iter()
                .try_fold(false, |acc, each_datum| {
                    Ok(acc || p.clone().run(&each_datum.0)?)
                })?,
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

pub fn glob(input: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return input == "";
    }

    // Parsing pattern
    let (saw_escape, mut patterns, mut working) = pattern.chars().fold(
        (false, vec![], "".to_string()),
        |(saw_escape, mut acc, mut working), c| {
            match c {
                '*' => {
                    if saw_escape {
                        working.push('*');
                        (false, acc, working)
                    } else {
                        acc.push(working);
                        working = "".to_string();
                        (false, acc, working)
                    }
                }
                '\\' => {
                    if saw_escape {
                        // Push prev escape
                        working.push('\\');
                    }
                    (true, acc, working)
                }
                _ => {
                    if saw_escape {
                        working.push('\\');
                    }

                    working.push(c);
                    (false, acc, working)
                }
            }
        },
    );

    if saw_escape {
        working.push('\\');
    }

    patterns.push(working);

    // Test input against the pattern
    patterns
        .iter()
        .enumerate()
        .try_fold(input, |acc, (idx, pattern_frag)| {
            if let Some((pre, post)) = acc.split_once(pattern_frag) {
                if idx == 0 && !pattern.starts_with("*") && !pre.is_empty() {
                    Err(())
                } else if idx == patterns.len() - 1 && !pattern.ends_with("*") && !post.is_empty() {
                    Err(())
                } else {
                    Ok(post)
                }
            } else {
                Err(())
            }
        })
        .is_ok()
}

impl TryFrom<Ipld> for Predicate {
    type Error = FromIpldError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::List(v) => match v.as_slice() {
                [Ipld::String(s), inner] if s == "not" => {
                    let inner = Box::new(Predicate::try_from(inner.clone())?);
                    Ok(Predicate::Not(inner))
                }
                [Ipld::String(op_str), Ipld::String(sel_str), val] => match op_str.as_str() {
                    "==" => {
                        let sel = Select::<ipld::Newtype>::from_str(sel_str.as_str())
                            .map_err(FromIpldError::InvalidIpldSelector)?;

                        Ok(Predicate::Equal(sel, ipld::Newtype(val.clone())))
                    }
                    ">" => {
                        let sel = Select::<ipld::Number>::from_str(sel_str.as_str())
                            .map_err(FromIpldError::InvalidNumberSelector)?;

                        let num = ipld::Number::try_from(val.clone())
                            .map_err(FromIpldError::CannotParseIpldNumber)?;

                        Ok(Predicate::GreaterThan(sel, num))
                    }
                    ">=" => {
                        let sel = Select::<ipld::Number>::from_str(sel_str.as_str())
                            .map_err(FromIpldError::InvalidNumberSelector)?;

                        let num = ipld::Number::try_from(val.clone())
                            .map_err(FromIpldError::CannotParseIpldNumber)?;
                        Ok(Predicate::GreaterThanOrEqual(sel, num))
                    }
                    "<" => {
                        let sel = Select::<ipld::Number>::from_str(sel_str.as_str())
                            .map_err(FromIpldError::InvalidNumberSelector)?;

                        let num = ipld::Number::try_from(val.clone())
                            .map_err(FromIpldError::CannotParseIpldNumber)?;

                        Ok(Predicate::LessThan(sel, num))
                    }
                    "<=" => {
                        let sel = Select::<ipld::Number>::from_str(sel_str.as_str())
                            .map_err(FromIpldError::InvalidNumberSelector)?;

                        let num = ipld::Number::try_from(val.clone())
                            .map_err(FromIpldError::CannotParseIpldNumber)?;

                        Ok(Predicate::LessThanOrEqual(sel, num))
                    }
                    "like" => {
                        let sel = Select::<String>::from_str(sel_str.as_str())
                            .map_err(FromIpldError::InvalidStringSelector)?;

                        if let Ipld::String(s) = val {
                            Ok(Predicate::Like(sel, s.to_string()))
                        } else {
                            Err(FromIpldError::NotAString(val.clone()))
                        }
                    }
                    "every" => {
                        let sel = Select::<ipld::Collection>::from_str(sel_str.as_str())
                            .map_err(FromIpldError::InvalidCollectionSelector)?;

                        let p = Box::new(Predicate::try_from(val.clone())?);
                        Ok(Predicate::Every(sel, p))
                    }
                    "some" => {
                        let sel = Select::<ipld::Collection>::from_str(sel_str.as_str())
                            .map_err(FromIpldError::InvalidCollectionSelector)?;

                        let p = Box::new(Predicate::try_from(val.clone())?);
                        Ok(Predicate::Some(sel, p))
                    }
                    _ => Err(FromIpldError::UnrecognizedTripleTag(op_str.to_string())),
                },
                [Ipld::String(op_str), lhs, rhs] => match op_str.as_str() {
                    "and" => {
                        let lhs = Box::new(Predicate::try_from(lhs.clone())?);
                        let rhs = Box::new(Predicate::try_from(rhs.clone())?);
                        Ok(Predicate::And(lhs, rhs))
                    }
                    "or" => {
                        let lhs = Box::new(Predicate::try_from(lhs.clone())?);
                        let rhs = Box::new(Predicate::try_from(rhs.clone())?);
                        Ok(Predicate::Or(lhs, rhs))
                    }
                    _ => Err(FromIpldError::UnrecognizedTripleTag(op_str.to_string())),
                },
                _ => Err(FromIpldError::UnrecognizedShape),
            },
            _ => Err(FromIpldError::NotATuple(ipld)),
        }
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum FromIpldError {
    #[error("Invalid Ipld selector {0:?}")]
    InvalidIpldSelector(<Select<ipld::Newtype> as FromStr>::Err),

    #[error("Invalid ipld::Number selector {0:?}")]
    InvalidNumberSelector(<Select<ipld::Number> as FromStr>::Err),

    #[error("Invalid ipld::Collection selector {0:?}")]
    InvalidCollectionSelector(<Select<ipld::Collection> as FromStr>::Err),

    #[error("Invalid String selector {0:?}")]
    InvalidStringSelector(<Select<ipld::Collection> as FromStr>::Err),

    #[error("Cannot parse ipld::Number {0:?}")]
    CannotParseIpldNumber(<ipld::Number as TryFrom<Ipld>>::Error),

    #[error("Not a string: {0:?}")]
    NotAString(Ipld),

    #[error("Unrecognized triple tag {0}")]
    UnrecognizedTripleTag(String),

    #[error("Unrecognized shape")]
    UnrecognizedShape,

    #[error("Not a predicate tuple {0:?}")]
    NotATuple(Ipld),
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
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_params: Self::Parameters) -> Self::Strategy {
        let leaf = prop_oneof![
            (Select::arbitrary(), ipld::Newtype::arbitrary())
                .prop_map(|(lhs, rhs)| { Predicate::Equal(lhs, rhs) }),
            (Select::arbitrary(), ipld::Number::arbitrary())
                .prop_map(|(lhs, rhs)| { Predicate::GreaterThan(lhs, rhs) }),
            (Select::arbitrary(), ipld::Number::arbitrary())
                .prop_map(|(lhs, rhs)| { Predicate::GreaterThanOrEqual(lhs, rhs) }),
            (Select::arbitrary(), ipld::Number::arbitrary())
                .prop_map(|(lhs, rhs)| { Predicate::LessThan(lhs, rhs) }),
            (Select::arbitrary(), ipld::Number::arbitrary())
                .prop_map(|(lhs, rhs)| { Predicate::LessThanOrEqual(lhs, rhs) }),
            (Select::arbitrary(), String::arbitrary())
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

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use pretty_assertions as pretty;
    use proptest::prelude::*;
    use testresult::TestResult;

    mod glob {
        use super::*;

        #[test_log::test]
        fn test_concrete() -> TestResult {
            let got = glob(&"hello world", &"hello world");
            assert!(got);
            Ok(())
        }

        #[test_log::test]
        fn test_concrete_fail() -> TestResult {
            let got = glob(&"hello world", &"NOPE");
            assert!(!got);
            Ok(())
        }

        #[test_log::test]
        fn test_empty_pattern_fail() -> TestResult {
            let got = glob(&"hello world", &"");
            assert!(!got);
            Ok(())
        }

        #[test_log::test]
        fn test_escaped_star() -> TestResult {
            let got = glob(&"*", &r#"\*"#);
            assert!(got);
            Ok(())
        }

        #[test_log::test]
        fn test_inner_escaped_star() -> TestResult {
            let got = glob(&"hello, * world*", &r#"hello*\**\*"#);
            assert!(got);
            Ok(())
        }

        #[test_log::test]
        fn test_empty_string_fail() -> TestResult {
            let got = glob(&"", &"NOPE");
            assert!(!got);
            Ok(())
        }

        #[test_log::test]
        fn test_left_star() -> TestResult {
            let got = glob(&"hello world", &"*world");
            assert!(got);
            Ok(())
        }

        #[test_log::test]
        fn test_left_star_failure() -> TestResult {
            let got = glob(&"hello world", &"*NOPE");
            assert!(!got);
            Ok(())
        }

        #[test_log::test]
        fn test_right_star() -> TestResult {
            let got = glob(&"hello world", &"hello*");
            assert!(got);
            Ok(())
        }

        #[test_log::test]
        fn test_right_star_failure() -> TestResult {
            let got = glob(&"hello world", &"NOPE*");
            assert!(!got);
            Ok(())
        }

        #[test_log::test]
        fn test_only_star() -> TestResult {
            let got = glob(&"hello world", &"*");
            assert!(got);
            Ok(())
        }

        #[test_log::test]
        fn test_two_stars() -> TestResult {
            let got = glob(&"hello world", &"* *");
            assert!(got);
            Ok(())
        }

        #[test_log::test]
        fn test_two_stars_fail() -> TestResult {
            let got = glob(&"hello world", &"*@*");
            assert!(!got);
            Ok(())
        }

        #[test_log::test]
        fn test_multiple_inner_stars() -> TestResult {
            let got = glob(&"hello world", &"h*l*o*w*r*d");
            assert!(got);
            Ok(())
        }

        #[test_log::test]
        fn test_multiple_inner_stars_fail() -> TestResult {
            let got = glob(&"hello world", &"a*b*c*d*e*f");
            assert!(!got);
            Ok(())
        }

        #[test_log::test]
        fn test_concrete_with_multiple_inner_stars() -> TestResult {
            let got = glob(&"hello world", &"hello* *world");
            assert!(got);
            Ok(())
        }
    }

    mod run {
        use super::*;
        use libipld::ipld;

        fn simple() -> Ipld {
            ipld!({
                "foo": 42,
                "bar": "baz".to_string(),
                "qux": true
            })
        }

        fn email() -> Ipld {
            ipld!({
                "from": "alice@example.com",
                "to": ["bob@example.com", "fraud@example.com"],
                "cc": ["carol@example.com"],
                "subject": "Quarterly Reports",
                "body": "Here's Q2 the reports ..."
            })
        }

        fn wasm() -> Ipld {
            ipld!({
                "mod": "data:application/wasm;base64,SOMEBASE64GOESHERE",
                "fun": "test",
                "input": [0, 1, 2 ,3]
            })
        }

        #[test_log::test]
        fn test_eq() -> TestResult {
            let p = Predicate::Equal(
                Select::from_str(".from").unwrap(),
                "alice@example.com".into(),
            );

            assert!(p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_eq_try_null() -> TestResult {
            let p = Predicate::Equal(Select::from_str(".not_from?").unwrap(), Ipld::Null.into());

            assert!(p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_eq_dot_field_ending_try_null() -> TestResult {
            let p = Predicate::Equal(Select::from_str(".from.not?").unwrap(), Ipld::Null.into());

            assert!(p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_eq_dot_field_inner_try_null() -> TestResult {
            let p = Predicate::Equal(Select::from_str(".nope?.not").unwrap(), Ipld::Null.into());

            assert!(p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_eq_root_try_not_null() -> TestResult {
            let p = Predicate::Equal(Select::from_str(".?").unwrap(), Ipld::Null.into());

            assert!(!p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_eq_try_not_null() -> TestResult {
            let p = Predicate::Equal(
                Select::from_str(".from?").unwrap(),
                "alice@example.com".into(),
            );

            assert!(p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_eq_nested_try_null() -> TestResult {
            let p = Predicate::Equal(Select::from_str(".from?.not?").unwrap(), Ipld::Null.into());

            assert!(p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_eq_fail_same_type() -> TestResult {
            let p = Predicate::Equal(Select::from_str(".from").unwrap(), "NOPE".into());

            assert!(!p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_eq_bad_selector() -> TestResult {
            let p = Predicate::Equal(
                Select::from_str(".NOPE").unwrap(),
                "alice@example.com".into(),
            );

            assert!(p.run(&email()).is_err());
            Ok(())
        }

        #[test_log::test]
        fn test_eq_fail_different_type() -> TestResult {
            let p = Predicate::Equal(Select::from_str(".from").unwrap(), 42.into());

            assert!(!p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_gt() -> TestResult {
            let p = Predicate::GreaterThan(Select::from_str(".foo").unwrap(), (41.9).into());
            assert!(p.run(&simple())?);
            Ok(())
        }

        #[test_log::test]
        fn test_gt_fail() -> TestResult {
            let p = Predicate::GreaterThan(Select::from_str(".foo").unwrap(), 42.into());
            assert!(!p.run(&simple())?);
            Ok(())
        }

        #[test_log::test]
        fn test_gte() -> TestResult {
            let p = Predicate::GreaterThanOrEqual(Select::from_str(".foo").unwrap(), 42.into());
            assert!(p.run(&simple())?);
            Ok(())
        }

        #[test_log::test]
        fn test_gte_fail() -> TestResult {
            let p = Predicate::GreaterThanOrEqual(Select::from_str(".foo").unwrap(), (42.1).into());
            assert!(!p.run(&simple())?);
            Ok(())
        }

        #[test_log::test]
        fn test_lt() -> TestResult {
            let p = Predicate::LessThan(Select::from_str(".foo").unwrap(), (42.1).into());
            assert!(p.run(&simple())?);
            Ok(())
        }

        #[test_log::test]
        fn test_lt_fail() -> TestResult {
            let p = Predicate::LessThan(Select::from_str(".foo").unwrap(), 42.into());
            assert!(!p.run(&simple())?);
            Ok(())
        }

        #[test_log::test]
        fn test_lte() -> TestResult {
            let p = Predicate::LessThanOrEqual(Select::from_str(".foo").unwrap(), 42.into());
            assert!(p.run(&simple())?);
            Ok(())
        }

        #[test_log::test]
        fn test_lte_fail() -> TestResult {
            let p = Predicate::LessThanOrEqual(Select::from_str(".foo").unwrap(), (41.9).into());
            assert!(!p.run(&simple())?);
            Ok(())
        }

        #[test_log::test]
        fn test_like() -> TestResult {
            let p = Predicate::Like(Select::from_str(".from").unwrap(), "alice@*".into());
            assert!(p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_like_fail_concrete() -> TestResult {
            let p = Predicate::Like(Select::from_str(".from").unwrap(), "NOPE".into());
            assert!(!p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_like_fail_left_star() -> TestResult {
            let p = Predicate::Like(Select::from_str(".from").unwrap(), "*NOPE".into());
            assert!(!p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_like_fail_right_star() -> TestResult {
            let p = Predicate::Like(Select::from_str(".from").unwrap(), "NOPE*".into());
            assert!(!p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_like_fail_both_stars() -> TestResult {
            let p = Predicate::Like(Select::from_str(".from").unwrap(), "*NOPE*".into());
            assert!(!p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_not() -> TestResult {
            let p = Predicate::Not(Box::new(Predicate::Equal(
                Select::from_str(".from").unwrap(),
                "NOPE".into(),
            )));

            assert!(p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_double_negative() -> TestResult {
            let p = Predicate::Not(Box::new(Predicate::Not(Box::new(Predicate::Equal(
                Select::from_str(".from").unwrap(),
                "alice@example.com".into(),
            )))));

            assert!(p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_not_fail() -> TestResult {
            let p = Predicate::Not(Box::new(Predicate::Equal(
                Select::from_str(".from").unwrap(),
                "alice@example.com".into(),
            )));

            assert!(!p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_and_both_succeed() -> TestResult {
            let p = Predicate::And(
                Box::new(Predicate::Equal(
                    Select::from_str(".from").unwrap(),
                    "alice@example.com".into(),
                )),
                Box::new(Predicate::Equal(
                    Select::from_str(".subject").unwrap(),
                    "Quarterly Reports".into(),
                )),
            );

            assert!(p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_and_left_fail() -> TestResult {
            let p = Predicate::And(
                Box::new(Predicate::Equal(
                    Select::from_str(".from").unwrap(),
                    "NOPE".into(),
                )),
                Box::new(Predicate::Equal(
                    Select::from_str(".subject").unwrap(),
                    "Quarterly Reports".into(),
                )),
            );

            assert!(!p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_and_right_fail() -> TestResult {
            let p = Predicate::And(
                Box::new(Predicate::Equal(
                    Select::from_str(".from").unwrap(),
                    "alice@example.com".into(),
                )),
                Box::new(Predicate::Equal(
                    Select::from_str(".subject").unwrap(),
                    "NOPE".into(),
                )),
            );

            assert!(!p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_and_both_fail() -> TestResult {
            let p = Predicate::And(
                Box::new(Predicate::Equal(
                    Select::from_str(".from").unwrap(),
                    "NOPE".into(),
                )),
                Box::new(Predicate::Equal(
                    Select::from_str(".subject").unwrap(),
                    "NOPE".into(),
                )),
            );

            assert!(!p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_or_both_succeed() -> TestResult {
            let p = Predicate::Or(
                Box::new(Predicate::Equal(
                    Select::from_str(".from").unwrap(),
                    "alice@example.com".into(),
                )),
                Box::new(Predicate::Equal(
                    Select::from_str(".subject").unwrap(),
                    "Quarterly Reports".into(),
                )),
            );

            assert!(p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_or_left_fail() -> TestResult {
            let p = Predicate::Or(
                Box::new(Predicate::Equal(
                    Select::from_str(".from").unwrap(),
                    "NOPE".into(),
                )),
                Box::new(Predicate::Equal(
                    Select::from_str(".subject").unwrap(),
                    "Quarterly Reports".into(),
                )),
            );

            assert!(p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_or_right_fail() -> TestResult {
            let p = Predicate::Or(
                Box::new(Predicate::Equal(
                    Select::from_str(".from").unwrap(),
                    "alice@example.com".into(),
                )),
                Box::new(Predicate::Equal(
                    Select::from_str(".subject").unwrap(),
                    "NOPE".into(),
                )),
            );

            assert!(p.run(&email())?);
            Ok(())
        }

        #[test_log::test]
        fn test_or_both_fail() -> TestResult {
            let p = Predicate::Or(
                Box::new(Predicate::Equal(
                    Select::from_str(".from").unwrap(),
                    "NOPE".into(),
                )),
                Box::new(Predicate::Equal(
                    Select::from_str(".subject").unwrap(),
                    "NOPE".into(),
                )),
            );

            assert!(!p.run(&email())?);
            Ok(())
        }

        // FIXME nested, too
        #[test_log::test]
        fn test_every() -> TestResult {
            let p = Predicate::Every(
                Select::from_str(".input[]").unwrap(),
                Box::new(Predicate::LessThan(
                    Select::from_str(".").unwrap(),
                    100.into(),
                )),
            );

            assert!(p.run(&wasm())?);
            Ok(())
        }

        #[test_log::test]
        fn test_every_failure() -> TestResult {
            let p = Predicate::Every(
                Select::from_str(".input[]").unwrap(),
                Box::new(Predicate::LessThan(
                    Select::from_str(".").unwrap(),
                    1.into(),
                )),
            );

            assert!(!p.run(&wasm())?);
            Ok(())
        }

        // FIXME nested, too
        #[test_log::test]
        fn test_some_all_succeed() -> TestResult {
            let p = Predicate::Some(
                Select::from_str(".input[]").unwrap(),
                Box::new(Predicate::LessThan(
                    Select::from_str(".").unwrap(),
                    100.into(),
                )),
            );

            assert!(p.run(&wasm())?);
            Ok(())
        }

        #[test_log::test]
        fn test_some_not_all() -> TestResult {
            let p = Predicate::Some(
                Select::from_str(".input[]").unwrap(),
                Box::new(Predicate::LessThan(
                    Select::from_str(".").unwrap(),
                    1.into(),
                )),
            );

            assert!(p.run(&wasm())?);
            Ok(())
        }

        #[test_log::test]
        fn test_some_all_fail() -> TestResult {
            let p = Predicate::Some(
                Select::from_str(".input[]").unwrap(),
                Box::new(Predicate::LessThan(
                    Select::from_str(".").unwrap(),
                    0.into(),
                )),
            );

            assert!(!p.run(&wasm())?);
            Ok(())
        }

        #[test_log::test]
        fn test_alternate_every_and_some() -> TestResult {
            // ["every", ".a", ["some", ".b[]", ["==", ".", 0]]]
            let p = Predicate::Every(
                Select::from_str(".a").unwrap(),
                Box::new(Predicate::Some(
                    Select::from_str(".b[]").unwrap(),
                    Box::new(Predicate::Equal(Select::from_str(".").unwrap(), 0.into())),
                )),
            );

            let nested_data = ipld!(
                {
                    "a": [
                        {
                            "b": {
                                "c": 0, // Yep
                                "d": 0, // Yep
                                "e": 1  // Nope, but ok because "some"
                            },
                            "not-b": "ignore"
                        },
                        {
                            "also-not-b": "ignore",
                            "b": [-1, 0, 1]
                        }
                    ]
                }
            );

            assert!(p.run(&nested_data)?);
            Ok(())
        }

        #[test_log::test]
        fn test_alternate_fail_every_and_some() -> TestResult {
            // ["every", ".a", ["some", ".b[]", ["==", ".", 0]]]
            let p = Predicate::Every(
                Select::from_str(".a").unwrap(),
                Box::new(Predicate::Some(
                    Select::from_str(".b[]").unwrap(),
                    Box::new(Predicate::Equal(Select::from_str(".").unwrap(), 0.into())),
                )),
            );

            let nested_data = ipld!(
                {
                    "a": [
                        {
                            "b": {
                                "c": 0, // Yep
                                "d": 0, // Yep
                                "e": 1  // Nope, but ok because "some"
                            },
                            "not-b": "ignore"
                        },
                        {
                            "also-not-b": "ignore",
                            "b": [-1, 42, 1] // No 0, so fail "every"
                        }
                    ]
                }
            );

            assert!(!p.run(&nested_data)?);
            Ok(())
        }

        // FIXME
        #[test_log::test]
        fn test_alternate_some_and_every() -> TestResult {
            // ["some", ".a", ["every", ".b[]", ["==", ".", 0]]]
            let p = Predicate::Some(
                Select::from_str(".a").unwrap(),
                Box::new(Predicate::Every(
                    Select::from_str(".b[]").unwrap(),
                    Box::new(Predicate::Equal(Select::from_str(".").unwrap(), 0.into())),
                )),
            );

            let nested_data = ipld!(
                {
                    "a": [
                        {
                            "b": {
                                "c": 0, // Yep
                                "d": 0, // Yep
                                "e": 1  // Nope, so fail this every, but...
                            },
                            "not-b": "ignore"
                        },
                        {
                            "also-not-b": "ignore",
                            "b": [0, 0, 0] // This every succeeds, so the outer "some" succeeds
                        }
                    ]
                }
            );

            assert!(p.run(&nested_data)?);
            Ok(())
        }

        // FIXME
        #[test_log::test]
        fn test_alternate_fail_some_and_every() -> TestResult {
            // ["some", ".a", ["every", ".b[]", ["==", ".", 0]]]
            let p = Predicate::Some(
                Select::from_str(".a").unwrap(),
                Box::new(Predicate::Every(
                    Select::from_str(".b[]").unwrap(),
                    Box::new(Predicate::Equal(Select::from_str(".").unwrap(), 0.into())),
                )),
            );

            let nested_data = ipld!(
                {
                    "a": [
                        {
                            "b": {
                                "c": 0, // Yep
                                "d": 0, // Yep
                                "e": 1  // Nope
                            },
                            "not-b": "ignore"
                        },
                        {
                            "also-not-b": "ignore",
                            "b": [-1, 42, 1] // Also nope, so fail
                        }
                    ]
                }
            );

            assert!(!p.run(&nested_data)?);
            Ok(())
        }

        #[test_log::test]
        fn test_deeply_alternate_some_and_every() -> TestResult {
            // ["some", ".a",
            //   ["every", ".b.c[]",
            //     ["some", ".d",
            //       ["every", ".e[]",
            //         ["==", ".f.g", 0]
            //       ]
            //     ]
            //   ]
            // ]
            let p = Predicate::Some(
                Select::from_str(".a").unwrap(),
                Box::new(Predicate::Every(
                    Select::from_str(".b.c[]").unwrap(),
                    Box::new(Predicate::Some(
                        Select::from_str(".d").unwrap(),
                        Box::new(Predicate::Every(
                            Select::from_str(".e[]").unwrap(),
                            Box::new(Predicate::Equal(
                                Select::from_str(".f.g").unwrap(),
                                0.into(),
                            )),
                        )),
                    )),
                )),
            );

            let deeply_nested_data = ipld!(
                {
                    // Some
                    "a": [
                        {
                            "b": {
                                "c": {
                                    // Every
                                    "c1": {
                                        // Some
                                        "d": [
                                            {
                                                // Every
                                                "e": {
                                                    "e1": {
                                                        "f": {
                                                            "g": 0
                                                        },
                                                        "nope": -10
                                                    },
                                                    "e2": {
                                                        "_": "not selected",
                                                        "f": {
                                                            "g": 0
                                                        },
                                                    }
                                                }
                                            }
                                        ]
                                    },
                                    "c2": {
                                        // Some
                                        "*": "avoid",
                                        "d": [
                                            {
                                                // Every
                                                "e": {
                                                    "e1": {
                                                        "f": {
                                                            "g": 0
                                                        },
                                                        "nope": -10
                                                    },
                                                    "e2": {
                                                        "_": "not selected",
                                                        "f": {
                                                            "g": 0
                                                        },
                                                    }
                                                }
                                            }
                                        ]
                                    }
                                }
                            }
                        }
                    ],
                    "z": "doesn't read this"
                }
            );

            assert!(p.run(&deeply_nested_data)?);
            Ok(())
        }
    }
}
