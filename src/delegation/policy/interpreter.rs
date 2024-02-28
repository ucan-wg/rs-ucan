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
pub struct Machine {
    pub args: arguments::Named<Ipld>,
    pub frames: BTreeMap<String, Stream>,
    pub program: Statement,
    pub index_counter: usize,
}

pub fn run(machine: Machine) -> Machine {
    // run to exhaustion
    // loop {
    //     if let Ok(next) = run_once(&machine) {
    //         if next == &machine {
    //             return machine;
    //         }
    //     } else {
    //         panic!("failed some step");
    //     }
    // }
    todo!()
}

pub fn run_once(mut context: Machine) -> Result<Machine, ()> {
    // FIXME Fix this iter; need to keep getting smaller and runninhg top-to-bottom
    // or at least that's one startegy
    match context.clone().program {
        Statement::And(left, right) => {
            let lhs = Machine {
                program: *left,
                ..context.clone()
            };

            let lhs_result = run(lhs);

            let rhs = Machine {
                args: context.args,
                frames: lhs_result.frames,
                program: *right,
                index_counter: lhs_result.index_counter,
            };

            let rhs_result = run(rhs);

            if rhs_result.frames.is_empty() {
                Err(())
            } else {
                Ok(rhs_result)
            }
        }
        Statement::Or(left, right) => {
            let lhs = Machine {
                program: *left,
                ..context.clone()
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
            let btree: BTreeMap<usize, Ipld> = match collection {
                Collection::Array(vec) => vec
                    .into_iter()
                    .map(|ipld| (context.next_index(), ipld))
                    .collect(),

                Collection::Map(map) => map
                    .into_iter()
                    .map(|(_k, ipld)| (context.next_index(), ipld))
                    .collect(),
            };

            context.frames.insert(var.0, Stream::Some(btree));
            Ok(context)
        }
        Statement::Forall(var, collection) => {
            let btree: BTreeMap<usize, Ipld> = match collection {
                Collection::Array(vec) => vec
                    .into_iter()
                    .map(|ipld| (context.next_index(), ipld))
                    .collect(),

                Collection::Map(map) => map
                    .into_iter()
                    .map(|(_k, ipld)| (context.next_index(), ipld))
                    .collect(),
            };

            context.frames.insert(var.0, Stream::Every(btree));

            // FIXME needs to check that nothing changed
            // ...perhaps at the end of the iteration, loop through the streams?

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
                let ipld = Ipld::Map(context.args.clone().0);
                let selected = select(selector, ipld)?;
                let idx = context.next_index();

                context
                    .frames
                    .insert(var.0, Stream::Every(BTreeMap::from_iter([(idx, selected)])));

                Ok(context)
            }
            SelectorValue::Literal(ipld) => {
                let ipld = select(selector, ipld)?;
                let idx = context.next_index();

                context
                    .frames
                    .insert(var.0, Stream::Every(BTreeMap::from_iter([(idx, ipld)])));

                Ok(context)
            }
            SelectorValue::Variable(var_id) => {
                let current = context
                    .frames
                    .get(&var_id.0)
                    .cloned()
                    .unwrap_or(Stream::Every(BTreeMap::new()));

                let result: Result<BTreeMap<usize, Ipld>, ()> = current
                    .clone()
                    .to_btree()
                    .into_iter()
                    .map(|(idx, ipld)| select(selector.clone(), ipld).map(|ipld| (idx, ipld)))
                    .collect();

                let updated = result?;

                context
                    .frames
                    .insert(var.0, current.map(|_| updated.clone()));

                Ok(context)
            }
        },
    }
}

pub fn select(selector: Selector, on: Ipld) -> Result<Ipld, ()> {
    let results: Vec<Ipld> = selector
        .0
        .iter()
        .try_fold(vec![on], |ipld_stream, segment| match segment {
            PathSegment::This => Ok(ipld_stream),
            PathSegment::Index(i) => {
                ipld_stream
                    .iter()
                    .try_fold(vec![], |mut acc, ipld_entry| match ipld_entry {
                        Ipld::List(vec) => {
                            if let Some(ipld) = vec.get(*i) {
                                acc.push(ipld.clone());
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
                                acc.push(ipld.clone());
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
                            acc.extend(vec.clone());
                            Ok(acc.iter().cloned().collect())
                        }
                        _ => Err(()),
                    })
            }
        })?;

    match &results[..] {
        [ipld] => Ok(ipld.clone()),
        vec => Ok(Ipld::List(
            vec.into_iter().map(|ipld| ipld.clone()).collect(),
        )),
    }
}

impl Machine {
    pub fn next_index(&mut self) -> usize {
        let prev = self.index_counter;
        self.index_counter += 1;
        prev
    }

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
                    let key = var_id.0.clone();

                    if let Some(stream) = self.frames.get(&key) {
                        let updated = stream.clone().map(|btree| {
                            btree
                                .into_iter()
                                .filter(|(_idx, ipld)| f(left_ipld, ipld))
                                .collect()
                        });

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
                let lhs_key = &var_id.0;

                if let Some(stream) = self.frames.get(lhs_key) {
                    match rhs {
                        Value::Literal(right_ipld) => {
                            let updated = stream.clone().map(|btree| {
                                btree
                                    .into_iter()
                                    .filter(|(_idx, ipld)| f(ipld, right_ipld))
                                    .collect()
                            });

                            if updated.is_empty() {
                                return Err(());
                            }

                            self.frames.insert(lhs_key.clone(), updated);
                            Ok(())
                        }
                        Value::Variable(var_id) => {
                            let rhs_key = var_id.0.as_str();

                            if let Some(stream) = self.frames.get(rhs_key) {
                                let updated = stream.clone().map(|btree| {
                                    btree
                                        .into_iter()
                                        .filter(|(_idx, ipld)| f(ipld, ipld))
                                        .collect()
                                });

                                if updated.is_empty() {
                                    return Err(());
                                }

                                self.frames.insert(lhs_key.clone(), updated);

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
