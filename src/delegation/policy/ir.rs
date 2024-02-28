use enum_as_inner::EnumAsInner;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Term {
    // Leaves
    Args, // $
    Literal(Ipld),
    Stream(Stream),

    Selector(Selector), // NOTE the IR version doens't inline the results

    // Connectives
    Not(Box<Term>),
    And(Vec<Term>),
    Or(Vec<Term>),

    // Comparison
    Equal(Value, Value), // AKA unification
    GreaterThan(Value, Value),
    LessThan(Value, Value),

    // String Matcher
    Glob(Value, Value),

    // Existential Quantification
    Forall(Variable, Collection), // ∀x ∈ xs
    Exists(Variable, Collection), // ∃x ∈ xs -> convert every -> some
}

// FIXME exract domain gen selectors first?
// FIXME rename constraint or validation or expression or something?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    // Connectives
    Not(Box<Statement>),
    And(Box<Statement>, Box<Statement>),
    Or(Box<Statement>, Box<Statement>),

    // Forall: unpack and unify size before and after
    // Exists genrates more than one frame (unpacks an array) instead of one
    Forall(Variable, Collection), // ∀x ∈ xs
    Exists(Variable, Collection), // ∃x ∈ xs

    // Comparison
    Equal(Value, Value), // AKA unification // FIXME value can also be a selector
    GreaterThan(Value, Value),
    GreaterThanOrEqual(Value, Value),
    LessThan(Value, Value),
    LessThanOrEqual(Value, Value),
    // >= and <= are probably good for efficiency

    // String Matcher
    Glob(Value, Value),

    // Select from ?foo
    Select(Selector, SelectorValue, Variable), // .foo.bar[].baz
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable(pub String); // ?x

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Collection {
    Array(Vec<Ipld>),
    Map(BTreeMap<String, Ipld>),
    // NOTE The below can always be desugared, esp because this is now only used with forall/exists
    // Variable(Variable), ]
    // Selector(Selector),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selector(pub Vec<PathSegment>); // .foo.bar[].baz

// FIXME need an IR representation of $args

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathSegment {
    This, // .
    // RecDesend,    // ..
    Index(usize), // [2]
    Key(String),  // ["key"] (or .key)
    FlattenAll,   // [] --> creates an Array
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelectorValue {
    Args,
    Literal(Ipld),
    Variable(Variable),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Literal(Ipld),
    Variable(Variable),
}

pub fn glob(input: &Ipld, pattern: &Ipld) -> bool {
    if let (Ipld::String(s), Ipld::String(pat)) = (input, pattern) {
        let mut input = s.chars();
        let mut pattern = pat.chars(); // Ugly

        loop {
            match (input.next(), pattern.next()) {
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
    panic!("FIXME");
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumAsInner)]
pub enum Stream {
    Every(BTreeMap<usize, Ipld>), // "All or nothing"
    Some(BTreeMap<usize, Ipld>),  // FIXME disambiguate from Option::Some
}

impl Stream {
    pub fn to_btree(self) -> BTreeMap<usize, Ipld> {
        match self {
            Stream::Every(xs) => xs,
            Stream::Some(xs) => xs,
        }
    }

    pub fn map(self, f: impl Fn(BTreeMap<usize, Ipld>) -> BTreeMap<usize, Ipld>) -> Stream {
        match self {
            Stream::Every(xs) => {
                let updated = f(xs);
                Stream::Every(updated)
            }
            Stream::Some(xs) => {
                let updated = f(xs);
                Stream::Some(updated)
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Stream::Every(xs) => xs.is_empty(),
            Stream::Some(xs) => xs.is_empty(),
        }
    }
}

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct EveryStream(V<Ipld>);
//
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct SomeStream(Vec<Ipld>);
