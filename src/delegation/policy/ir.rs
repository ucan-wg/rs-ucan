//FIXME rename core
use enum_as_inner::EnumAsInner;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Term {
    // Leaves
    Literal(Ipld),
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
    Every(Variable, Value), // ∀x ∈ xs
    Some(Variable, Value),  // ∃x ∈ xs -> convert every -> some
}

// FIXME exract domain gen selectors first?
// FIXME rename constraint or validation or expression or something?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    // Select from ?foo
    Select(Selector, SelectorValue, Variable), // .foo.bar[].baz

    // Connectives
    Not(Box<Statement>),
    And(Box<Statement>, Box<Statement>),
    Or(Box<Statement>, Box<Statement>),

    // String Matcher
    Glob(Value, Value),
    Equal(Value, Value), // AKA unification // FIXME value can also be a selector

    // Comparison
    GreaterThan(Value, Value),
    GreaterThanOrEqual(Value, Value),
    LessThan(Value, Value),
    LessThanOrEqual(Value, Value),

    // Forall: unpack and unify size before and after
    // Exists genrates more than one frame (unpacks an array) instead of one
    Forall(Variable, Collection), // ∀x ∈ xs
    Exists(Variable, Collection), // ∃x ∈ xs
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Variable(pub String); // ?x

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Collection {
    Array(Vec<Ipld>),
    Map(BTreeMap<String, Ipld>),
    Selector(Vec<Ipld>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selector(pub Vec<PathSegment>); // .foo.bar[].baz

// FIXME need an IR representation of $args

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathSegment {
    This,         // .
    Index(usize), // [2]
    Key(String),  // ["key"] (or .key)
                  // FlattenAll,   // [] --> creates an Array
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelectorValue {
    Literal(Ipld),
    Variable(Variable),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Literal(Ipld),
    Selector(Vec<PathSegment>),
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
    pub fn remove(&mut self, key: usize) {
        match self {
            Stream::Every(xs) => {
                xs.remove(&key);
            }
            Stream::Some(xs) => {
                xs.remove(&key);
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Stream::Every(xs) => xs.len(),
            Stream::Some(xs) => xs.len(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&usize, &Ipld)> {
        match self {
            Stream::Every(xs) => xs.iter(),
            Stream::Some(xs) => xs.iter(),
        }
    }

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogicIpld {
    Null,
    Bool(bool),
    Float(f64),
    Integer(i128),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<LogicIpld>),
    Map(BTreeMap<String, LogicIpld>),

    // A new challenger has appeared!
    Var(String),
}

impl LogicIpld {
    pub fn substitute(&mut self, bindings: BTreeMap<Variable, LogicIpld>) {
        match self {
            LogicIpld::Var(var_id) => {
                if let Some(value) = bindings.get(&Variable(*var_id)) {
                    *self = value.clone();
                }
            }
            LogicIpld::List(xs) => {
                for x in xs {
                    x.substitute(bindings.clone());
                }
            }
            LogicIpld::Map(btree) => {
                for (_, x) in btree {
                    x.substitute(bindings.clone());
                }
            }
            _other => (),
        }
    }

    pub fn extract_into(&self, vars: &mut BTreeSet<Variable>) {
        match self {
            LogicIpld::Var(var_id) => {
                vars.insert(Variable(var_id.clone()));
            }
            LogicIpld::List(xs) => {
                for x in xs {
                    x.extract_into(vars);
                }
            }
            LogicIpld::Map(btree) => {
                for (_, x) in btree {
                    x.extract_into(vars);
                }
            }
            _other => (),
        }
    }

    pub fn unify(
        self,
        other: Self,
        bindings: BTreeMap<Variable, LogicIpld>,
    ) -> Result<(Self, BTreeSet<Variable>), ()> {
        match (self, other) {
            (LogicIpld::Null, LogicIpld::Null) => Ok((LogicIpld::Null, BTreeSet::new())),
            (LogicIpld::Bool(a), LogicIpld::Bool(b)) => {
                if a == b {
                    Ok((LogicIpld::Bool(a), BTreeSet::new()))
                } else {
                    Err(())
                }
            }
            (LogicIpld::Float(a), LogicIpld::Float(b)) => {
                if a == b {
                    Ok((LogicIpld::Float(a), BTreeSet::new()))
                } else {
                    Err(())
                }
            }
            (LogicIpld::Integer(a), LogicIpld::Integer(b)) => {
                if a == b {
                    Ok((LogicIpld::Integer(a), BTreeSet::new()))
                } else {
                    Err(())
                }
            }
            (LogicIpld::String(a), LogicIpld::String(b)) => {
                if a == b {
                    Ok((LogicIpld::String(a), BTreeSet::new()))
                } else {
                    Err(())
                }
            }
            (LogicIpld::Bytes(a), LogicIpld::Bytes(b)) => {
                if a == b {
                    Ok((LogicIpld::Bytes(a), BTreeSet::new()))
                } else {
                    Err(())
                }
            }
            (LogicIpld::List(a), LogicIpld::List(b)) => {
                // FIXME
                if a.len() != b.len() {
                    return Err(());
                }
                let mut bindings = BTreeSet::new();
                let mut result = Vec::with_capacity(a.len());
                for (a, b) in a.into_iter().zip(b.into_iter()) {
                    let (unified, mut new_bindings) = a.unify(b)?;
                    result.push(unified);
                    bindings.append(&mut new_bindings);
                }
                Ok((LogicIpld::List(result), bindings))
            }
            (LogicIpld::Map(a), LogicIpld::Map(b)) => {
                // FIXME
                if a.len() != b.len() {
                    return Err(());
                }
                let mut bindings = BTreeSet::new();
                let mut result = BTreeMap::new();
                for (k, a) in a.into_iter() {
                    if let Some(b) = b.get(&k) {
                        let (unified, mut new_bindings) = a.unify(b.clone())?;
                        result.insert(k, unified);
                        bindings.append(&mut new_bindings);
                    } else {
                        return Err(());
                    }
                }
                Ok((LogicIpld::Map(result), bindings))
            }
            (LogicIpld::Var(a), LogicIpld::Var(b)) => {
                // FIXME

                // If I have a binding for a, and no binding for b, set ?b := ?a
                // If I have a binding for b, and no binding for a, set ?a := ?b
                // If I have neither binding, what do???
                // If I have both bindings, and they are the same, great
                //     1. check ?a == ?b
                //     2 set ?a := ?b
                //           NOTE: would need to update the lookup procedure elsewhere
                //                 to do recursive lookup
                // If I have both bindings, and they are not immeditely equal,
                //   recursively unify them, and then set ?a := ?b
                //   NOTE: during recursion, if we see ?a in ?b or ?b in ?a, you need to bail

                if a == b {
                    Ok((LogicIpld::Var(a), BTreeSet::new()))
                } else {
                    Err(())
                }
            }
            (LogicIpld::Var(lhs_tag), logic_ipld) => {
                match logic_ipld {
                    rhs @ LogicIpld::Var(rhs_tag) => {
                        //FIXME double check
                        if let Some(b) = bindings.get(&rhs_tag) {
                            let (unified, mut new_bindings) =
                                LogicIpld::Var(a).unify(rhs.clone())?;
                            new_bindings.insert(rhs.clone());
                            Ok((unified, new_bindings))
                        } else {
                            let mut new_bindings = BTreeSet::new();
                            new_bindings.insert(Variable(b.clone()));
                            Ok((LogicIpld::Var(a), new_bindings))
                        }
                    }

                    _ => {
                        let mut new_bindings = BTreeSet::new();
                        new_bindings.insert(Variable(a.clone()));
                        Ok((logic_ipld, new_bindings))
                    }
                }
            }
            (lhs, rhs @ LogicIpld::Var(_)) => rhs.unify(lhs, bindings),

            _ => Err(()),
        }
    }
}

impl TryFrom<LogicIpld> for Ipld {
    type Error = Variable;

    fn try_from(logic: LogicIpld) -> Result<Ipld, Self::Error> {
        match logic {
            LogicIpld::Null => Ok(Ipld::Null),
            LogicIpld::Bool(b) => Ok(Ipld::Bool(b)),
            LogicIpld::Float(f) => Ok(Ipld::Float(f)),
            LogicIpld::Integer(i) => Ok(Ipld::Integer(i)),
            LogicIpld::String(s) => Ok(Ipld::String(s)),
            LogicIpld::Bytes(b) => Ok(Ipld::Bytes(b)),
            LogicIpld::List(xs) => xs
                .into_iter()
                .try_fold(vec![], |mut acc, x| {
                    acc.push(Ipld::try_from(x)?);
                    Ok(acc)
                })
                .map(Ipld::List),
            LogicIpld::Map(btree) => btree
                .into_iter()
                .try_fold(BTreeMap::new(), |mut acc, (k, v)| {
                    acc.insert(k, Ipld::try_from(v)?);
                    Ok(acc)
                })
                .map(Ipld::Map),
            LogicIpld::Var(var_id) => Err(Variable(var_id)),
        }
    }
}
