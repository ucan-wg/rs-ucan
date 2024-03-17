pub mod filter;

mod error;
mod select;
mod selectable;

pub use error::{ParseError, SelectorErrorReason};
pub use select::Select;
pub use selectable::Selectable;

use crate::ipld;
use filter::Filter;
use libipld_core::ipld::Ipld;
use nom::{
    self,
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::map_res,
    error::context,
    multi::{many0, many1},
    sequence::preceded,
    IResult,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::{fmt, str::FromStr};
use thiserror::Error;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Selector(pub Vec<Filter>);

impl Selector {
    pub fn new() -> Self {
        Selector(vec![])
    }

    pub fn is_related(&self, other: &Selector) -> bool {
        self.0.iter().zip(other.0.iter()).all(|(a, b)| a == b)
    }

    //     pub fn get(&self, ctx: &Ipld) -> Result<ipld::Newtype, SelectorError> {
    //         let ipld: Ipld = self
    //             .0
    //             .iter()
    //             .try_fold((ctx.clone(), vec![]), |(ipld, mut seen_ops), op| {
    //                 seen_ops.push(op);
    //
    //                 match op {
    //                     Filter::Try(inner) => {
    //                         let op: Filter = *inner.clone();
    //                         let ipld: Ipld = Select::Get::<Ipld>(vec![op])
    //                             .resolve(ctx)
    //                             .unwrap_or(Ipld::Null);
    //
    //                         Ok((ipld, seen_ops))
    //                     }
    //                     Filter::ArrayIndex(i) => {
    //                         let result = {
    //                             match ipld {
    //                                 Ipld::List(xs) => {
    //                                     if i.abs() as usize > xs.len() {
    //                                         return Err(SelectorError::from_refs(
    //                                             &seen_ops,
    //                                             SelectorErrorReason::IndexOutOfBounds,
    //                                         ));
    //                                     };
    //
    //                                     xs.get((xs.len() as i32 + *i) as usize)
    //                                         .ok_or(SelectorError::from_refs(
    //                                             &seen_ops,
    //                                             SelectorErrorReason::IndexOutOfBounds,
    //                                         ))
    //                                         .cloned()
    //                                 }
    //                                 // FIXME behaviour on maps? type error
    //                                 _ => Err(SelectorError::from_refs(
    //                                     &seen_ops,
    //                                     SelectorErrorReason::NotAList,
    //                                 )),
    //                             }
    //                         };
    //
    //                         Ok((result?, seen_ops))
    //                     }
    //                     Filter::Field(k) => {
    //                         let result = match ipld {
    //                             Ipld::Map(xs) => xs
    //                                 .get(k)
    //                                 .ok_or(SelectorError::from_refs(
    //                                     &seen_ops,
    //                                     SelectorErrorReason::KeyNotFound,
    //                                 ))
    //                                 .cloned(),
    //                             _ => Err(SelectorError::from_refs(
    //                                 &seen_ops,
    //                                 SelectorErrorReason::NotAMap,
    //                             )),
    //                         };
    //
    //                         Ok((result?, seen_ops))
    //                     }
    //                     Filter::Values => {
    //                         let result = match ipld {
    //                             Ipld::List(xs) => Ok(Ipld::List(xs)),
    //                             Ipld::Map(xs) => Ok(Ipld::List(xs.values().cloned().collect())),
    //                             _ => Err(SelectorError::from_refs(
    //                                 &seen_ops,
    //                                 SelectorErrorReason::NotACollection,
    //                             )),
    //                         };
    //
    //                         Ok((result?, seen_ops))
    //                     }
    //                 }
    //             })?
    //             .0;
    //
    //         Ok(ipld::Newtype(ipld))
    //     }
}

pub fn parse(input: &str) -> IResult<&str, Selector> {
    let without_this = many1(filter::parse);
    let with_this = preceded(char('.'), many0(filter::parse));

    // NOTE: must try without_this this first, to disambiguate `.field` from `.`
    let p = map_res(alt((without_this, with_this)), |found| {
        Ok::<Selector, ()>(Selector(found))
    });

    context("selector", p)(input)
}

pub fn parse_this(input: &str) -> IResult<&str, Selector> {
    let p = map_res(tag("."), |_| Ok::<Selector, ()>(Selector(vec![])));
    context("this", p)(input)
}

pub fn parse_selector_ops(input: &str) -> IResult<&str, Vec<Filter>> {
    let p = many1(filter::parse);
    context("filters", p)(input)
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ops = self.0.iter();

        if let Some(field) = ops.next() {
            if !field.is_dot_field() {
                write!(f, ".")?;
            }

            write!(f, "{}", field)?;
        } else {
            write!(f, ".")?;
        }

        for op in ops {
            op.fmt(f)?;
        }

        Ok(())
    }
}

impl FromStr for Selector {
    type Err = nom::Err<ParseError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with(".") {
            return Err(nom::Err::Error(ParseError::MissingStartingDot(
                s.to_string(),
            )));
        }

        if s.len() == 0 {
            return Err(nom::Err::Error(ParseError::MissingStartingDot(
                s.to_string(),
            )));
        }

        let working;
        let mut acc = vec![];

        if let Ok((more, found)) = filter::parse_dot_field(s) {
            working = more;
            acc.push(found);
        } else {
            working = &s[1..];
        }

        match many0(filter::parse)(working) {
            Ok(("", ops)) => {
                let mut mut_ops = ops.clone();
                acc.append(&mut mut_ops);
                Ok(Selector(acc))
            }
            Ok((more, _ops)) => Err(nom::Err::Error(ParseError::TrailingInput(more.to_string()))),
            Err(err) => Err(err.map(|input| ParseError::UnknownPattern(input.to_string()))),
        }
    }
}
impl Serialize for Selector {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Selector {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Selector::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
#[error("Selector {selector} encountered runtime error: {reason}")]
pub struct SelectorError {
    pub selector: Selector,
    pub reason: SelectorErrorReason,
}

impl SelectorError {
    pub fn from_refs(path_refs: &Vec<&Filter>, reason: SelectorErrorReason) -> SelectorError {
        SelectorError {
            selector: Selector(path_refs.iter().map(|op| (*op).clone()).collect()),
            reason,
        }
    }
}

impl PartialOrd for Selector {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self == other {
            return Some(Ordering::Equal);
        }

        if self.0.starts_with(&other.0) {
            return Some(Ordering::Greater);
        }

        if other.0.starts_with(&self.0) {
            return Some(Ordering::Less);
        }

        None
    }
}

#[cfg(feature = "test_utils")]
impl Arbitrary for Selector {
    type Parameters = <Filter as Arbitrary>::Parameters;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        prop::collection::vec(Filter::arbitrary_with(args), 0..12)
            .prop_map(|ops| Selector(ops))
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    mod serialization {
        use super::*;

        proptest! {
            #[test]
            fn test_selector_round_trip(sel: Selector) {
                let serialized = sel.to_string();
                let deserialized = serialized.parse();
                prop_assert_eq!(Ok(sel), deserialized);
            }
        }
    }
}
