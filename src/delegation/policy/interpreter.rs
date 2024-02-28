use super::ir::*;
use crate::ability::arguments;
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;

// [and ["==", ".foo", "?x"]
//      [">",  "?x", 0]
//      [">",  "?x", 2]
//      ["==", 10, 11] // Fails, so whole thing fails?
//      ["or", ["==", ".bar", "?y"]
//             [">",  "?y",    12]
//             ["and", ["<", "?x", 100]
//                     ["<", "?y", 100]
//             ]
//             ["every", "?x", "?e"]
//      ]
//      ["==", 22, "?e"]
//      ["some", "?x" "?a"]
//      ["==", "?a", [1, 2, "?z", 4]]
//      ["==", ["?b", "?c", 20, 30], [10, "?a", 20, 30]] // -> b = 10, c = a, a = c
// ]

// [".[]", "$", ?x]
// [".[]", "$", ?y]
// ["==", "?x", "?y"]
// ["==", "?y", "?x"]

// Register machine
// {
//   ports: {
//     "?a": Stream<>,
//     "?b": Stream<>,
//   }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct Machine<'a> {
    pub args: arguments::Named<Ipld>,
    pub frames: BTreeMap<&'a str, Stream>,
    pub program: Statement,
    pub index_counter: usize,
}

pub fn run<'a>(machine: Machine<'a>) -> Machine<'a> {
    // run to exhaustion
    loop {
        if let Ok(next) = run_once(&machine) {
            if next == &machine {
                return machine;
            }
        } else {
            panic!("failed some step");
        }
    }
}

pub fn run_once<'a>(mut context: Machine<'a>) -> Result<Machine<'a>, ()> {
    // FIXME Fix this iter; need to keep getting smaller and runninhg top-to-bottom
    // or at least that's one startegy
    match context.program {
        Statement::And(left, right) => {
            let lhs = Machine {
                program: *left,
                ..context
            };

            let lhs_result = run(lhs);

            let rhs = Machine {
                args: context.args,
                frames: lhs_result.frames,
                program: *right,
                index_counter: lhs_result.index_counter,
            };

            let mut rhs_result = run(rhs);

            if rhs_result.frames.is_empty() {
                Err(())
            } else {
                Ok(rhs_result)
            }
        }
        Statement::Or(left, right) => {
            let lhs = Machine {
                program: *left,
                ..context
            };

            let rhs = Machine {
                program: *right,
                ..context
            };

            let lhs_result = run(lhs);
            let rhs_result = run(rhs);
            todo!() // merge_and_dedup(lhs_result, rhs_result);
        }
        Statement::Not(statement) => {
            let next = Machine {
                args: context.args,
                frames: context.frames,
                program: *statement,
                index_counter: context.index_counter,
            };

            let not_results = run(next);

            todo!(); // remove all not_results from context.frames
        }
        Statement::Exists(var, collection) => {
            let xs: Vec<Ipld> = match collection {
                Collection::Array(vec) => vec,
                Collection::Map(map) => map.values().cloned().collect(),
            };

            context.frames.insert(var.0.as_str(), Stream::Some(xs));
            Ok(context)
        }
        Statement::Forall(var, collection) => {
            let xs: Vec<Ipld> = match collection {
                Collection::Array(vec) => vec,
                Collection::Map(map) => map.values().cloned().collect(),
            };

            context.frames.insert(var.0.as_str(), Stream::Every(xs));
            Ok(context)
        }
        Statement::Equal(left, right) => context
            .apply(&left, &right, |a, b| a == b)
            .map(|()| context),
        Statement::GreaterThan(left, right) => context
            .apply(&left, &right, |a, b| match (a, b) {
                (Ipld::Integer(a), Ipld::Integer(b)) => a > b,
                (Ipld::Float(a), Ipld::Float(b)) => a > b,
                (Ipld::Integer(a), Ipld::Float(b)) => (*a as f64) > *b,
                (Ipld::Float(a), Ipld::Integer(b)) => *a > (*b as f64),
                _ => false,
            })
            .map(|()| context),
        Statement::LessThan(left, right) => context
            .apply(&left, &right, |a, b| match (a, b) {
                (Ipld::Integer(a), Ipld::Integer(b)) => a < b,
                (Ipld::Float(a), Ipld::Float(b)) => a < b,
                (Ipld::Integer(a), Ipld::Float(b)) => (*a as f64) < *b,
                (Ipld::Float(a), Ipld::Integer(b)) => *a < (*b as f64),
                _ => false,
            })
            .map(|()| context),
        Statement::GreaterThanOrEqual(left, right) => context
            .apply(&left, &right, |a, b| match (a, b) {
                (Ipld::Integer(a), Ipld::Integer(b)) => a >= b,
                (Ipld::Float(a), Ipld::Float(b)) => a >= b,
                (Ipld::Integer(a), Ipld::Float(b)) => (*a as f64) >= *b,
                (Ipld::Float(a), Ipld::Integer(b)) => *a >= (*b as f64),
                _ => false,
            })
            .map(|()| context),

        Statement::LessThanOrEqual(left, right) => context
            .apply(&left, &right, |a, b| match (a, b) {
                (Ipld::Integer(a), Ipld::Integer(b)) => a <= b,
                (Ipld::Float(a), Ipld::Float(b)) => a <= b,
                (Ipld::Integer(a), Ipld::Float(b)) => (*a as f64) <= *b,
                (Ipld::Float(a), Ipld::Integer(b)) => *a <= (*b as f64),
                _ => false,
            })
            .map(|()| context),

        Statement::Glob(left, right) => context
            .apply(&left, &right, |a, b| glob(a, b))
            .map(|()| context),
        Statement::Select(selector, target, var) => match target {
            SelectorValue::Args => {
                let ipld = Ipld::Map(context.args.0);
                let selected = select(selector, ipld)?;

                context
                    .frames
                    .insert(var.0.as_str(), Stream::Every(vec![selected]));

                Ok(context)
            }
            SelectorValue::Literal(ipld) => {
                let ipld = select(selector, ipld)?;

                context
                    .frames
                    .insert(var.0.as_str(), Stream::Every(vec![ipld]));

                Ok(context)
            }
            SelectorValue::Variable(var_id) => {
                let current = context
                    .frames
                    .get(var_id.0.as_str())
                    .unwrap_or(&Stream::Every(vec![]));

                let result: Result<Vec<Ipld>, ()> = current
                    .to_vec()
                    .iter()
                    .map(|ipld| select(selector.clone(), ipld.clone()))
                    .collect();

                let updated = result?;
                current.map(|_| updated);

                Ok(context)
            }
        },
    }
}

pub fn select(selector: Selector, on: Ipld) -> Result<Ipld, ()> {
    let results: Vec<&Ipld> =
        selector
            .0
            .iter()
            .try_fold(vec![&on], |mut ipld_stream, segment| match segment {
                PathSegment::This => Ok(ipld_stream),
                PathSegment::Index(i) => {
                    ipld_stream
                        .iter()
                        .try_fold(vec![], |mut acc, ipld_entry| match ipld_entry {
                            Ipld::List(vec) => {
                                if let Some(ipld) = vec.get(*i) {
                                    acc.push(ipld);
                                    Ok(acc)
                                } else {
                                    Err(())
                                }
                            }
                            _ => Err(()),
                        })
                }
                PathSegment::Key(key) => {
                    ipld_stream
                        .iter()
                        .try_fold(vec![], |mut acc, ipld_entry| match ipld_entry {
                            Ipld::Map(map) => {
                                if let Some(ipld) = map.get(key) {
                                    acc.push(ipld);
                                    Ok(acc)
                                } else {
                                    Err(())
                                }
                            }
                            _ => Err(()),
                        })
                }
                PathSegment::FlattenAll => {
                    ipld_stream
                        .iter()
                        .try_fold(vec![], |mut acc, ipld_entry| match ipld_entry {
                            Ipld::List(vec) => {
                                acc.extend(vec);
                                Ok(acc)
                            }
                            _ => Err(()),
                        })
                }
            })?;

    match results.as_slice() {
        [ipld] => Ok(*ipld.clone()),
        vec => Ok(Ipld::List(
            vec.into_iter().map(|ipld| *ipld.clone()).collect(),
        )),
    }
}

// pub fn select_step(segment: PathSegment, ipld: Ipld) -> Result<Ipld, ()> {
//     match segment {
//         PathSegment::This => Ok(ipld),
//         PathSegment::Index(i) => match ipld {
//             Ipld::List(vec) => vec.get(i).cloned().ok_or(()),
//             _ => Err(()),
//         },
//         PathSegment::Key(key) => match ipld {
//             Ipld::Map(map) => map.get(&key).cloned().ok_or(()),
//             _ => Err(()),
//         },
//         PathSegment::FlattenAll => todo!(),
//     }
// }

impl<'a> Machine<'a> {
    pub fn apply<F>(&mut self, lhs: &Value, rhs: &Value, f: F) -> Result<(), ()>
    where
        F: Fn(&Ipld, &Ipld) -> bool,
    {
        match lhs {
            Value::Literal(left_ipld) => match rhs {
                Value::Literal(right_ipld) => {
                    if f(left_ipld, right_ipld) {
                        Ok(())
                    } else {
                        Err(())
                    }
                }
                Value::Variable(var_id) => {
                    let key = var_id.0.as_str();
                    if let Some(stream) = self.frames.get(key) {
                        let updated = stream
                            .map(|vec| vec.into_iter().filter(|ipld| f(left_ipld, ipld)).collect());

                        if updated.is_empty() {
                            return Err(());
                        }

                        self.frames.insert(key, updated);

                        Ok(())
                    } else {
                        Err(())
                    }
                }
            },
            Value::Variable(var_id) => {
                let lhs_key = var_id.0.as_str();
                if let Some(stream) = self.frames.get(lhs_key) {
                    match rhs {
                        Value::Literal(right_ipld) => {
                            let updated = stream.map(|vec| {
                                vec.into_iter().filter(|ipld| f(ipld, right_ipld)).collect()
                            });

                            if updated.is_empty() {
                                return Err(());
                            }

                            self.frames.insert(lhs_key, updated);
                            Ok(())
                        }
                        Value::Variable(var_id) => {
                            let rhs_key = var_id.0.as_str();
                            if let Some(stream) = self.frames.get(rhs_key) {
                                let updated = stream.map(|vec| {
                                    vec.into_iter().filter(|ipld| f(ipld, ipld)).collect()
                                });

                                if updated.is_empty() {
                                    return Err(());
                                }

                                self.frames.insert(lhs_key, updated);

                                Ok(())
                            } else {
                                Err(())
                            }
                        }
                    }
                } else {
                    // FIXME not nessesarily! You may need to create new entires
                    Err(())
                }
            }
        }
    }

    pub fn unify(&mut self, lhs: &Value, rhs: &Value) -> Result<Value, ()> {
        self.apply(lhs, rhs, |a, b| {
            todo!();
            todo!();
            // FIXME pattern matching etc
        })
        .map(|()| rhs.clone())
    }
}
