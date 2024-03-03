//FIXME rename core
use super::selector::op::SelectorOp;
use crate::ipld;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

impl Predicate {
    pub fn run(self, data: &Ipld) -> Result<bool, SelectorError> {
        Ok(match self {
            Predicate::True => true,
            Predicate::False => false,
            Predicate::Equal(lhs, rhs) => lhs.resolve(data)? == rhs.resolve(data)?,
            Predicate::GreaterThan(lhs, rhs) => lhs.resolve(data)? > rhs.resolve(data)?,
            Predicate::GreaterThanOrEqual(lhs, rhs) => lhs.resolve(data)? >= rhs.resolve(data)?,
            Predicate::LessThan(lhs, rhs) => lhs.resolve(data)? < rhs.resolve(data)?,
            Predicate::LessThanOrEqual(lhs, rhs) => lhs.resolve(data)? <= rhs.resolve(data)?,
            Predicate::Like(lhs, rhs) => glob(&lhs.resolve(data)?, &rhs.resolve(data)?),
            Predicate::Not(inner) => !inner.run(data)?,
            Predicate::And(lhs, rhs) => lhs.run(data)? && rhs.run(data)?,
            Predicate::Or(lhs, rhs) => lhs.run(data)? || rhs.run(data)?,
            Predicate::Forall(xs, p) => xs
                .resolve(data)?
                .to_vec()
                .iter()
                .try_fold(true, |acc, ipld| Ok(acc && p.clone().run(ipld)?))?,
            Predicate::Exists(xs, p) => {
                let pred = p.clone();

                xs.resolve(data)?
                    .to_vec()
                    .iter()
                    .try_fold(true, |acc, ipld| Ok(acc || pred.clone().run(ipld)?))?
            }
        })
    }
}

trait Resolve<T> {
    fn resolve(self, ctx: &Ipld) -> Result<T, SelectorError>;
}

impl Resolve<Ipld> for Ipld {
    fn resolve(self, _ctx: &Ipld) -> Result<Ipld, SelectorError> {
        Ok(self)
    }
}

impl Resolve<ipld::Number> for ipld::Number {
    fn resolve(self, _ctx: &Ipld) -> Result<ipld::Number, SelectorError> {
        Ok(self)
    }
}

impl Resolve<Collection> for Collection {
    fn resolve(self, _ctx: &Ipld) -> Result<Collection, SelectorError> {
        Ok(self)
    }
}

// FIXME Normal form?

impl Resolve<String> for String {
    fn resolve(self, _ctx: &Ipld) -> Result<String, SelectorError> {
        Ok(self)
    }
}

impl<T: TryFromIpld> SelectorOr<T> {
    fn resolve(self, ctx: &Ipld) -> Result<T, SelectorError> {
        match self {
            SelectorOr::Pure(inner) => Ok(inner),
            SelectorOr::Get(ops) => {
                ops.iter()
                    .try_fold((ctx.clone(), vec![]), |(ipld, mut seen_ops), op| {
                        seen_ops.push(op);

                        match op {
                            //                             SelectorOp::This => Ok((ipld, seen_ops)),
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
                                                return Err(SelectorError {
                                                    path: seen_ops
                                                        .iter()
                                                        .map(|op| (*op).clone())
                                                        .collect(),
                                                    reason: SelectorErrorReason::IndexOutOfBounds,
                                                });
                                            }

                                            xs.get((xs.len() as i32 + *i) as usize)
                                                .ok_or(SelectorError {
                                                    path: seen_ops
                                                        .iter()
                                                        .map(|op| (*op).clone())
                                                        .collect(),
                                                    reason: SelectorErrorReason::IndexOutOfBounds,
                                                })
                                                .cloned()
                                        }
                                        // FIXME behaviour on maps? type error
                                        _ => Err(SelectorError {
                                            path: seen_ops.iter().map(|op| (*op).clone()).collect(),
                                            reason: SelectorErrorReason::NotAList,
                                        }),
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

                                Ok((result?.clone(), seen_ops))
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

                                Ok((result?.clone(), seen_ops))
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

pub trait TryFromIpld: Sized {
    fn try_from_ipld(ipld: Ipld) -> Result<Self, SelectorErrorReason>;
}

impl TryFromIpld for Ipld {
    fn try_from_ipld(ipld: Ipld) -> Result<Ipld, SelectorErrorReason> {
        Ok(ipld)
    }
}

impl TryFromIpld for ipld::Number {
    fn try_from_ipld(ipld: Ipld) -> Result<ipld::Number, SelectorErrorReason> {
        match ipld {
            Ipld::Integer(i) => Ok(ipld::Number::Integer(i)),
            Ipld::Float(f) => Ok(ipld::Number::Float(f)),
            _ => Err(SelectorErrorReason::NotANumber),
        }
    }
}

impl TryFromIpld for String {
    fn try_from_ipld(ipld: Ipld) -> Result<Self, SelectorErrorReason> {
        match ipld {
            Ipld::String(s) => Ok(s),
            _ => Err(SelectorErrorReason::NotAString),
        }
    }
}

impl TryFromIpld for Collection {
    fn try_from_ipld(ipld: Ipld) -> Result<Collection, SelectorErrorReason> {
        match ipld {
            Ipld::List(xs) => Ok(Collection::Array(xs)),
            Ipld::Map(xs) => Ok(Collection::Map(xs)),
            _ => Err(SelectorErrorReason::NotACollection),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectorError {
    pub path: Vec<SelectorOp>,
    pub reason: SelectorErrorReason,
}

impl SelectorError {
    pub fn from_refs(path_refs: &Vec<&SelectorOp>, reason: SelectorErrorReason) -> SelectorError {
        SelectorError {
            path: path_refs.iter().map(|op| (*op).clone()).collect(),
            reason,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectorErrorReason {
    IndexOutOfBounds,
    KeyNotFound,
    NotAList,
    NotAMap,
    NotACollection,
    NotANumber,
    NotAString,
}

// FIXME exract domain gen selectors first?
// FIXME rename constraint or validation or expression or something?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Predicate {
    // Booleans
    True,
    False,

    // Comparison
    Equal(SelectorOr<Ipld>, SelectorOr<Ipld>),

    GreaterThan(SelectorOr<ipld::Number>, SelectorOr<ipld::Number>),
    GreaterThanOrEqual(SelectorOr<ipld::Number>, SelectorOr<ipld::Number>),

    LessThan(SelectorOr<ipld::Number>, SelectorOr<ipld::Number>),
    LessThanOrEqual(SelectorOr<ipld::Number>, SelectorOr<ipld::Number>),

    Like(SelectorOr<String>, SelectorOr<String>),

    // Connectives
    Not(Box<Predicate>),
    And(Box<Predicate>, Box<Predicate>),
    Or(Box<Predicate>, Box<Predicate>),

    // Collection iteration
    Forall(SelectorOr<Collection>, Box<Predicate>), // ∀x ∈ xs
    Exists(SelectorOr<Collection>, Box<Predicate>), // ∃x ∈ xs
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelectorOr<T> {
    Get(Vec<SelectorOp>),
    Pure(T),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Collection {
    Array(Vec<Ipld>),
    Map(BTreeMap<String, Ipld>),
}

impl Collection {
    pub fn to_vec(self) -> Vec<Ipld> {
        match self {
            Collection::Array(xs) => xs,
            Collection::Map(xs) => xs.into_values().collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Selector(pub Vec<SelectorOp>);

impl Serialize for Selector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0
            .iter()
            .fold("".into(), |acc, seg| format!("{}{}", acc, seg.to_string()))
            .serialize(serializer)
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
