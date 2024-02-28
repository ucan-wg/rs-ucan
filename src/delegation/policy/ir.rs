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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    // Connectives
    Not(Box<Statement>),
    And(Vec<Statement>),
    Or(Vec<Statement>),

    // Comparison
    Equal(Value, Value), // AKA unification
    GreaterThan(Value, Value),
    LessThan(Value, Value),

    // String Matcher
    Glob(Value, Value),

    Forall(Variable, Collection), // ∀x ∈ xs
    Exists(Variable, Collection), // ∃x ∈ xs
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable(pub String); // ?x

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Collection {
    Array(Vec<Ipld>),
    Map(BTreeMap<String, Ipld>),
    Variable(Variable),
    Selector(Selector),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selector(pub Vec<PathSegment>); // .foo.bar[].baz

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathSegment {
    This, // .
    // RecDesend,    // ..
    FlattenAll,   // .[] --> creates an Every stream
    Index(usize), // .[2]
    Key(String),  // .key
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stream {
    Every(Vec<Ipld>), // "All or nothing"
    Some(Vec<Ipld>),
}

impl Stream {
    pub fn is_empty(&self) -> bool {
        match self {
            Stream::Every(xs) => xs.is_empty(),
            Stream::Some(xs) => xs.is_empty(),
        }
    }
}

pub struct EveryStream(Vec<Ipld>);
pub struct SomeStream(Vec<Ipld>);

pub trait Apply<T> {
    // FIXME -> Option<(Stream, Stream)>?
    fn apply<F>(&self, other: &T, f: F) -> (Stream, Stream)
    where
        F: Fn(&Ipld, &Ipld) -> bool;

    fn better_apply<F>(&self, other: &T, f: F) -> Result<(Stream, Stream), ()>
    where
        F: Fn(&Ipld, &Ipld) -> bool,
    {
        if self.apply(other, f).0.is_empty() {
            Err(())
        } else {
            Ok((self, other))
        }
    }
}

impl Apply<Ipld> for Ipld {
    fn apply<F>(&self, other: &Ipld, f: F) -> (Stream, Stream)
    where
        F: Fn(&Ipld, &Ipld) -> bool,
    {
        if f(self, other) {
            (
                Stream::Every(vec![self.clone()]),
                Stream::Every(vec![other.clone()]),
            )
        } else {
            (Stream::Every(vec![]), Stream::Every(vec![]))
        }
    }
}

impl Apply<EveryStream> for Ipld {
    fn apply<F>(&self, other: &EveryStream, f: F) -> (Stream, Stream)
    where
        F: Fn(&Ipld, &Ipld) -> bool,
    {
        let mut y_results = vec![];

        for y in other.0.iter() {
            if f(self, y) {
                y_results.push(y.clone());
            } else {
                y_results = vec![];
                break;
            }
        }

        if y_results.is_empty() {
            (Stream::Every(vec![]), Stream::Every(vec![]))
        } else {
            (Stream::Every(vec![self.clone()]), Stream::Every(y_results))
        }
    }
}

impl Apply<Ipld> for EveryStream {
    fn apply<F>(&self, other: &Ipld, f: F) -> (Stream, Stream)
    where
        F: Fn(&Ipld, &Ipld) -> bool,
    {
        let mut x_results = vec![];

        for x in self.0.iter() {
            if f(x, other) {
                x_results.push(x.clone());
            } else {
                x_results = vec![];
                break;
            }
        }

        if x_results.is_empty() {
            (Stream::Every(vec![]), Stream::Every(vec![]))
        } else {
            (Stream::Every(x_results), Stream::Every(vec![other.clone()]))
        }
    }
}

impl Apply<Ipld> for SomeStream {
    fn apply<F>(&self, other: &Ipld, f: F) -> (Stream, Stream)
    where
        F: Fn(&Ipld, &Ipld) -> bool,
    {
        let mut x_results = vec![];

        for x in self.0.iter() {
            if f(x, other) {
                x_results.push(x.clone());
            }
        }

        (Stream::Some(x_results), Stream::Every(vec![other.clone()]))
    }
}

impl Apply<SomeStream> for Ipld {
    fn apply<F>(&self, other: &SomeStream, f: F) -> (Stream, Stream)
    where
        F: Fn(&Ipld, &Ipld) -> bool,
    {
        let mut y_results = vec![];

        for y in other.0.iter() {
            if f(self, y) {
                y_results.push(y.clone());
            } else {
                y_results = vec![];
                break;
            }
        }

        (Stream::Every(vec![self.clone()]), Stream::Some(y_results))
    }
}

impl Apply<EveryStream> for EveryStream {
    fn apply<F>(&self, other: &EveryStream, f: F) -> (Stream, Stream)
    where
        F: Fn(&Ipld, &Ipld) -> bool,
    {
        let mut x_results = vec![];
        let mut y_results = vec![];

        for x in self.0.iter() {
            for y in other.0.iter() {
                if f(x, y) {
                    x_results.push(x.clone());
                    y_results.push(y.clone());
                } else {
                    x_results = vec![];
                    y_results = vec![];
                    break;
                }
            }
        }

        (Stream::Every(x_results), Stream::Every(y_results))
    }
}

// FIXME
impl Apply<SomeStream> for EveryStream {
    fn apply<F>(&self, other: &SomeStream, f: F) -> (Stream, Stream)
    where
        F: Fn(&Ipld, &Ipld) -> bool,
    {
        let mut x_results = vec![];
        let mut y_results = vec![];

        for x in self.0.iter() {
            for y in other.0.iter() {
                if f(x, y) {
                    x_results.push(x.clone());
                    y_results.push(y.clone());
                } else {
                    x_results = vec![];
                    y_results.push(y.clone());
                    break;
                }
            }
        }

        (Stream::Every(x_results), Stream::Some(y_results))
    }
}

impl Apply<EveryStream> for SomeStream {
    fn apply<F>(&self, other: &EveryStream, f: F) -> (Stream, Stream)
    where
        F: Fn(&Ipld, &Ipld) -> bool,
    {
        let mut x_results = vec![];
        let mut y_results = vec![];

        for x in self.0.iter() {
            for y in other.0.iter() {
                if f(x, y) {
                    x_results.push(x.clone());
                    y_results.push(y.clone());
                } else {
                    x_results = vec![];
                    y_results.push(y.clone());
                    break;
                }
            }
        }

        (Stream::Some(x_results), Stream::Every(y_results))
    }
}

impl Apply<SomeStream> for SomeStream {
    fn apply<F>(&self, other: &SomeStream, f: F) -> (Stream, Stream)
    where
        F: Fn(&Ipld, &Ipld) -> bool,
    {
        let mut x_results = vec![];
        let mut y_results = vec![];

        for x in self.0.iter() {
            for y in other.0.iter() {
                if f(x, y) {
                    x_results.push(x.clone());
                    y_results.push(y.clone());
                }
            }
        }

        (Stream::Some(x_results), Stream::Some(y_results))
    }
}

impl Apply<Stream> for Stream {
    fn apply<F>(&self, other: &Stream, f: F) -> (Stream, Stream)
    where
        F: Fn(&Ipld, &Ipld) -> bool,
    {
        match self {
            Stream::Every(xs) => match other {
                Stream::Every(ys) => EveryStream(xs.clone()).apply(&EveryStream(ys.clone()), f),
                Stream::Some(ys) => EveryStream(xs.clone()).apply(&EveryStream(ys.clone()), f),
            },

            Stream::Some(xs) => match other {
                Stream::Every(ys) => SomeStream(xs.clone()).apply(&EveryStream(ys.clone()), f),
                Stream::Some(ys) => SomeStream(xs.clone()).apply(&SomeStream(ys.clone()), f),
            },
        }
    }
}

impl Apply<Ipld> for Stream {
    fn apply<F>(&self, other: &Ipld, f: F) -> (Stream, Stream)
    where
        F: Fn(&Ipld, &Ipld) -> bool,
    {
        todo!()
        // match self {
        //     Stream::Every(xs) => EveryStream(xs).apply(&other, f)
        //     Stream::Some(xs) => SomeStream(xs).apply(&other, f),
        // }
    }
}

impl Apply<Stream> for Ipld {
    fn apply<F>(&self, other: &Stream, f: F) -> (Stream, Stream)
    where
        F: Fn(&Ipld, &Ipld) -> bool,
    {
        todo!()
    }
}

impl Apply<Value> for Value {
    fn apply<F>(&self, other: &Value, f: F) -> (Stream, Stream)
    where
        F: Fn(&Ipld, &Ipld) -> bool,
    {
        todo!()
    }
}

// impl Stream {
//     /// Call like stream.apply(other_stream, |x, y| x == y)
//     pub fn apply<F>(&self, other: &Stream, f: F) -> (Stream, Stream)
//     where
//         F: Fn(&Ipld, &Ipld) -> bool,
//     {
//         match self {
//             Stream::Every(xs) => match other {
//                 Stream::Every(ys) => {
//                     let mut x_results = Vec::new();
//                     let mut y_results = Vec::new();
//
//                     for x in xs {
//                         for y in ys {
//                             if f(x, y) {
//                                 x_results.push(x.clone());
//                                 y_results.push(y.clone());
//                             } else {
//                                 x_results = vec![];
//                                 y_results = vec![];
//                                 break;
//                             }
//                         }
//                     }
//
//                     (Stream::Every(x_results), Stream::Every(y_results))
//                 }
//                 Stream::Some(ys) => {
//                     let mut x_results = Vec::new();
//                     let mut y_results = Vec::new();
//
//                     for x in xs {
//                         for y in ys {
//                             if f(x, y) {
//                                 x_results.push(x.clone());
//                                 y_results.push(y.clone());
//                             } else {
//                                 x_results = vec![];
//                                 break;
//                             }
//                         }
//                     }
//
//                     if &Stream::Every(x_results.clone()) == self {
//                         (Stream::Every(x_results), Stream::Some(y_results))
//                     } else {
//                         (Stream::Every(vec![]), Stream::Some(y_results))
//                     }
//                 }
//             },
//
//             Stream::Some(xs) => match other {
//                 Stream::Every(ys) => {
//                     let mut x_results = Vec::new();
//                     let mut y_results = Vec::new();
//
//                     for x in xs {
//                         for y in ys {
//                             if f(x, y) {
//                                 x_results.push(x.clone());
//                                 y_results.push(x.clone());
//                             } else {
//                                 x_results.push(x.clone());
//                                 y_results = vec![];
//                                 break;
//                             }
//                         }
//                     }
//
//                     (Stream::Some(x_results), Stream::Every(y_results))
//                 }
//                 Stream::Some(ys) => {
//                     let mut x_results = Vec::new();
//                     let mut y_results = Vec::new();
//
//                     for x in xs {
//                         for y in ys {
//                             if f(x, y) {
//                                 x_results.push(x.clone());
//                                 y_results.push(y.clone());
//                             }
//                         }
//                     }
//
//                     (Stream::Some(x_results), Stream::Some(y_results))
//                 }
//             },
//         }
//     }
// }
