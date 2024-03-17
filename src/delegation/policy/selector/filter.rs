use super::error::ParseError;
use enum_as_inner::EnumAsInner;
use nom::{
    self,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, anychar, char, digit1},
    combinator::{map_opt, map_res},
    error::context,
    multi::many1,
    sequence::{delimited, preceded, terminated},
    IResult,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumAsInner)]
pub enum Filter {
    ArrayIndex(i32),  // [2]
    Field(String),    // ["key"] (or .key)
    Values,           // .[]
    Try(Box<Filter>), // ?
}

impl Filter {
    pub fn is_in(&self, other: &Self) -> bool {
        match (self, other) {
            (Filter::ArrayIndex(a), Filter::ArrayIndex(b)) => a == b,
            (Filter::Field(a), Filter::Field(b)) => a == b,
            (Filter::Values, Filter::Values) => true,
            (Filter::ArrayIndex(_a), Filter::Values) => true,
            (Filter::Field(_k), Filter::Values) => true,
            (Filter::Try(a), Filter::Try(b)) => a.is_in(b), // FIXME Try is basically == null?
            _ => false,
        }
    }

    pub fn is_dot_field(&self) -> bool {
        match self {
            Filter::Field(k) => {
                if let Some(first) = k.chars().next() {
                    (first.is_alphabetic() || first == '_')
                        && k.chars().all(|c| char::is_alphanumeric(c) || c == '_')
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Filter::ArrayIndex(i) => write!(f, "[{}]", i),
            Filter::Field(k) => {
                if self.is_dot_field() {
                    write!(f, ".{}", k)
                } else {
                    write!(f, "[\"{}\"]", k)
                }
            }
            Filter::Values => write!(f, "[]"),
            Filter::Try(inner) => write!(f, "{}?", inner),
        }
    }
}

pub fn parse(input: &str) -> IResult<&str, Filter> {
    let p = alt((parse_try, parse_non_try));
    context("selector_op", p)(input)
}

pub fn parse_non_try(input: &str) -> IResult<&str, Filter> {
    let p = alt((parse_values, parse_field, parse_array_index));
    context("non_try", p)(input)
}

pub fn parse_try(input: &str) -> IResult<&str, Filter> {
    let p = map_res(terminated(parse_non_try, tag("?")), |found: Filter| {
        Ok::<Filter, ()>(Filter::Try(Box::new(found)))
    });

    context("try", p)(input)
}

pub fn parse_array_index(input: &str) -> IResult<&str, Filter> {
    let num = nom::combinator::recognize(preceded(nom::combinator::opt(tag("-")), digit1));

    let array_index = map_res(delimited(char('['), num, char(']')), |found| {
        let idx = i32::from_str(found).map_err(|_| ())?;
        Ok::<Filter, ()>(Filter::ArrayIndex(idx))
    });

    context("array_index", array_index)(input)
}

pub fn parse_values(input: &str) -> IResult<&str, Filter> {
    context("values", tag("[]"))(input).map(|(rest, _)| (rest, Filter::Values))
}

pub fn parse_field(input: &str) -> IResult<&str, Filter> {
    let p = alt((parse_delim_field, parse_dot_field));

    context("map_field", p)(input)
}

pub fn parse_dot_field(input: &str) -> IResult<&str, Filter> {
    let p = alt((parse_dot_alpha_field, parse_dot_underscore_field));
    context("dot_field", p)(input)
}

fn dot_starter(input: &str) -> IResult<&str, &str> {
    if input.len() < 2 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let bytes = input.as_bytes();

    if bytes[0] == b'.' {
        if char::from(bytes[1]).is_alphabetic() || bytes[1] == b'_' {
            return Ok((&input[2..], &input[..2]));
        }
    }

    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

fn is_allowed_in_dot_field(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

pub fn parse_dot_alpha_field(input: &str) -> IResult<&str, Filter> {
    let p = map_res(
        preceded(
            dot_starter,
            nom::multi::many0(nom::character::complete::satisfy(is_allowed_in_dot_field)),
        ),
        |found: Vec<char>| {
            let inner = [input.as_bytes()[1] as char]
                .iter()
                .chain(found.iter())
                .collect::<String>();
            Ok::<Filter, ()>(Filter::Field(inner))
        },
    );

    context("dot_field", p)(input)
}

pub fn parse_dot_underscore_field(input: &str) -> IResult<&str, Filter> {
    let p = map_res(preceded(tag("._"), alphanumeric1), |found: &str| {
        let key = format!("{}{}", '_', found);
        Ok::<Filter, ()>(Filter::Field(key))
    });

    context("dot_field", p)(input)
}

pub fn parse_empty_quotes_field(input: &str) -> IResult<&str, Filter> {
    let p = map_res(tag("[\"\"]"), |_: &str| {
        Ok::<Filter, ()>(Filter::Field("".to_string()))
    });

    context("empty_quotes_field", p)(input)
}

pub fn unicode_or_space(input: &str) -> IResult<&str, &str> {
    #[derive(Copy, Clone, PartialEq, Debug)]
    enum Status {
        Looking,
        FoundQuote,
        Done,
        Failed,
    }

    let (status, len) =
        input
            .as_bytes()
            .iter()
            .fold((Status::Looking, 0), |(status, len), byte| {
                if status == Status::Failed {
                    return (status, len);
                }

                if status == Status::Done {
                    return (status, len);
                }

                let c = char::from(*byte);

                if status == Status::FoundQuote {
                    if c == ']' {
                        return (Status::Done, len + 1);
                    } else {
                        return (Status::Looking, len + 1);
                    }
                }

                if c == '"' {
                    return (Status::FoundQuote, len + 1);
                }

                if c == ' ' || (!nom_unicode::is_whitespace(c) && !nom_unicode::is_control(c)) {
                    return (Status::Looking, len + 1);
                }

                (Status::Failed, 0)
            });

    match (status, len) {
        (Status::Done, len) => Ok((&input[len - 2..], &input[..len - 2])),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::TakeWhile1,
        ))),
    }
}

pub fn parse_delim_field(input: &str) -> IResult<&str, Filter> {
    let p = map_res(
        delimited(tag(r#"[""#), unicode_or_space, tag(r#""]"#)),
        |found: &str| Ok::<Filter, ()>(Filter::Field(found.to_string())),
    );

    context("delimited_field", alt((p, parse_empty_quotes_field)))(input)
}

impl FromStr for Filter {
    type Err = nom::Err<ParseError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse(s).map_err(|e| nom::Err::Failure(ParseError::UnknownPattern(e.to_string())))? {
            (_, found) => Ok(found),
            // ("", found) => Ok(found),
            // FIXME
            // (rest, _) => Err(nom::Err::Failure(ParseError::TrailingInput(rest.into()))),
        }
    }
}
impl Serialize for Filter {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Filter {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Filter::from_str(&s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

#[cfg(feature = "test_utils")]
impl Arbitrary for Filter {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_params: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            i32::arbitrary().prop_map(|i| Filter::ArrayIndex(i)),
            "[a-zA-Z_ ]*".prop_map(Filter::Field),
            "[a-zA-Z_][a-zA-Z0-9_]*".prop_map(Filter::Field),
            Just(Filter::Values),
            // FIXME prop_recursive::lazy(|_| { Filter::arbitrary_with(()).prop_map(Filter::Try) }),
        ]
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
            fn test_filter_round_trip(filter: Filter) {
                let serialized = filter.to_string();
                let deserialized = serialized.parse();
                prop_assert_eq!(Ok(filter), deserialized);
            }
        }
    }
}
