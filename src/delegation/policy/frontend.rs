use super::ir;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Term {
    // Leaves
    Args, // $
    Literal(Ipld),
    Variable(Variable),

    Selector(Selector),

    // Connectives
    Not(Box<Term>),
    And(Vec<Term>),
    Or(Vec<Term>),

    // Comparison
    Equal(Value, Value),
    GreaterThan(Value, Value),
    GreaterOrEqual(Value, Value),
    LessThan(Value, Value),
    LessOrEqual(Value, Value),

    // String Matcher
    Glob(Value, String),

    // Existential Quantification
    Exists(Variable, Collection), // ∃x ∈ xs
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable(String); // ?x

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Collection {
    Array(Vec<Ipld>),
    Map(BTreeMap<String, Ipld>),
    Variable(Variable),
    Selector(Selector),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selector(Vec<Index>); // .foo.bar[].baz

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Index {
    This,
    // RecDesend,    // ..
    FlattenAll,   // .[]
    Index(usize), // .[2]
    Key(String),  // .key
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Literal(Ipld),
    Variable(Variable),
    ImplicitBind(Selector),
}
