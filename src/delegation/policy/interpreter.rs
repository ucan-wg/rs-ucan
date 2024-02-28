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
    todo!()
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
}

pub fn step(mut context: Machine) -> Result<Machine, ()> {
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
                ..context.clone()
            };

            let lhs_result = run(lhs);
            let rhs_result = run(rhs);

            let merged_frames = lhs_result
                .frames
                .into_iter()
                .map(|(key, lhs_stream)| {
                    let rhs_stream = rhs_result
                        .frames
                        .get(&key)
                        .cloned()
                        .unwrap_or(lhs_stream.clone());

                    let merged = match (lhs_stream, rhs_stream) {
                        (Stream::Every(lhs), Stream::Every(rhs)) => {
                            Stream::Every(lhs.into_iter().chain(rhs).collect())
                        }
                        (Stream::Some(lhs), Stream::Some(rhs)) => {
                            Stream::Some(lhs.into_iter().chain(rhs).collect())
                        }
                        (Stream::Every(lhs), Stream::Some(rhs)) => {
                            Stream::Every(lhs.into_iter().chain(rhs).collect())
                        }
                        (Stream::Some(lhs), Stream::Every(rhs)) => {
                            Stream::Every(lhs.into_iter().chain(rhs).collect())
                        }
                    };

                    (key, merged)
                })
                .collect();

            Ok(Machine {
                frames: merged_frames,
                ..context
            })
        }
        Statement::Not(statement) => {
            let next = Machine {
                program: *statement,
                ..context.clone()
            };

            let not_results = run(next);

            for (idx, _) in not_results.frames.iter() {
                context.frames.remove(idx);
            }

            Ok(context)
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
        Statement::Equal(left, right) => context // FIXME do unification
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

                        match updated {
                            Stream::Every(ref btree) => {
                                if btree.len() < stream.len() {
                                    return Err(());
                                }
                            }
                            Stream::Some(ref btree) => {
                                if btree.is_empty() {
                                    return Err(());
                                }
                            }
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

                if let Some(lhs_stream) = self.frames.get(lhs_key) {
                    match rhs {
                        Value::Literal(right_ipld) => {
                            let updated = lhs_stream.clone().map(|btree| {
                                btree
                                    .into_iter()
                                    .filter(|(_idx, ipld)| f(ipld, right_ipld))
                                    .collect()
                            });

                            match updated {
                                Stream::Every(ref btree) => {
                                    if btree.len() < lhs_stream.len() {
                                        return Err(());
                                    }
                                }
                                Stream::Some(ref btree) => {
                                    if btree.is_empty() {
                                        return Err(());
                                    }
                                }
                            }

                            self.frames.insert(lhs_key.clone(), updated);
                            Ok(())
                        }
                        Value::Variable(var_id) => {
                            let rhs_key = var_id.0.as_str();

                            if let Some(rhs_stream) = self.frames.get(rhs_key) {
                                let mut non_matches: BTreeMap<usize, Vec<usize>> = BTreeMap::new();

                                for (lhs_id, lhs_value) in lhs_stream.iter() {
                                    for (rhs_id, rhs_value) in rhs_stream.iter() {
                                        if !f(lhs_value, rhs_value) {
                                            if let Some(rhs_ids) = non_matches.get_mut(lhs_id) {
                                                rhs_ids.push(*rhs_id);
                                            } else {
                                                non_matches.insert(*lhs_id, vec![*rhs_id]);
                                            }
                                        }
                                    }
                                }

                                // Double negatives, but for good reason
                                let did_quantify =
                                    match (lhs_stream.is_every(), rhs_stream.is_every()) {
                                        (true, true) => non_matches.is_empty(),
                                        (true, false) => non_matches
                                            .values()
                                            .all(|rhs_ids| rhs_ids.len() != rhs_stream.len()),
                                        (false, true) => {
                                            non_matches.values().any(|rhs_ids| rhs_ids.is_empty())
                                        }
                                        (false, false) => non_matches
                                            .values()
                                            .any(|rhs_ids| rhs_ids.len() < rhs_stream.len()),
                                    };

                                if did_quantify {
                                    let mut new_lhs_stream = lhs_stream.clone();
                                    let mut new_rhs_stream = rhs_stream.clone();

                                    for (l_key, r_keys) in non_matches {
                                        new_lhs_stream.remove(l_key);

                                        for r_key in r_keys {
                                            new_rhs_stream.remove(r_key);
                                        }
                                    }

                                    self.frames.insert(lhs_key.into(), new_lhs_stream);
                                    self.frames.insert(rhs_key.into(), new_rhs_stream);

                                    Ok(())
                                } else {
                                    Err(())
                                }
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

    pub fn pattern_matching_unification(&mut self, lhs: &Value, rhs: &Value) -> Result<Value, ()> {
        self.apply(lhs, rhs, |a, b| {
            todo!();
            todo!();
            // FIXME pattern matching etc
        })
        .map(|()| rhs.clone())
    }
}
