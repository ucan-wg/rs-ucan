pub mod error;
pub mod op;

use error::ParseError;
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
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub struct Selector(Vec<op::SelectorOp>);

pub fn parse(input: &str) -> IResult<&str, Selector> {
    let without_this = many1(op::parse);
    let with_this = preceded(char('.'), many0(op::parse));

    // NOTE: must try without this first, to disambiguate `.field` from `.`
    let p = map_res(alt((without_this, with_this)), |found| {
        Ok::<Selector, ()>(Selector(found))
    });

    context("selector", p)(input)
}

pub fn parse_this(input: &str) -> IResult<&str, Selector> {
    let p = map_res(tag("."), |_| Ok::<Selector, ()>(Selector(vec![])));
    context("this", p)(input)
}

pub fn parse_selector_ops(input: &str) -> IResult<&str, Vec<op::SelectorOp>> {
    let p = many1(op::parse);
    context("selector ops", p)(input)
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ops = self.0.iter();

        if let Some(field) = ops.next() {
            field.fmt(f)?;
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
        match parse(s).map_err(|e| nom::Err::Failure(ParseError::UnknownPattern(e.to_string())))? {
            ("", selector) => Ok(selector),
            (rest, _) => Err(nom::Err::Failure(ParseError::TrailingInput(rest.into()))),
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
