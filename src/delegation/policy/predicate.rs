use super::selector::{Select, SelectorError};
use crate::ipld;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};

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
    Equal(Select<ipld::Newtype>, Select<ipld::Newtype>),

    GreaterThan(Select<ipld::Number>, Select<ipld::Number>),
    GreaterThanOrEqual(Select<ipld::Number>, Select<ipld::Number>),

    LessThan(Select<ipld::Number>, Select<ipld::Number>),
    LessThanOrEqual(Select<ipld::Number>, Select<ipld::Number>),

    Like(Select<String>, Select<String>),

    // Connectives
    Not(Box<Predicate>),
    And(Box<Predicate>, Box<Predicate>),
    Or(Box<Predicate>, Box<Predicate>),

    // Collection iteration
    Every(Select<ipld::Collection>, Box<Predicate>), // ∀x ∈ xs
    Some(Select<ipld::Collection>, Box<Predicate>),  // ∃x ∈ xs
}

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
