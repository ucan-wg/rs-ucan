use super::ir::*;
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;

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
    pub ports: BTreeMap<&'a str, Stream>,
    pub program: BTreeMap<&'a str, Statement>,
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

pub fn run_once<'a>(machine: &'a Machine<'a>) -> Result<&'a Machine<'a>, ()> {
    let mut ports = machine.ports.clone();
    let mut program = machine.program.clone();

    // FIXME Fix this iter; need to keep getting smaller and runninhg top-to-bottom
    // or at least that's one startegy
    program
        .clone()
        .iter()
        .try_fold((), |acc, (idx, statement)| {
            // FIXME change from map to vec
            match statement {
                Statement::Glob(value, pattern) => value
                    .better_apply(&pattern, &glob)
                    .map(|_| program.remove(idx)),
                Statement::Equal(left, right) => left
                    .better_apply(right, PartialEq::eq)
                    .map(|_| program.remove(idx)),
                Statement::GreaterThan(left, right) => left
                    .better_apply(right, PartialEq::gt)
                    .map(|_| program.remove(idx)),
                Statement::LessThan(left, right) => left
                    .better_apply(right, PartialEq::lt)
                    .map(|_| program.remove(idx)),
            }
            // Statement::Equal(left, right) => match (left, right) {
            //     (Value::Literal(left), Value::Literal(right)) => {
            //         if left == right {
            //             program.remove(idx);
            //             Ok(())
            //         } else {
            //             Err(())
            //         }
            //     }
            //     (Value::Literal(left), Value::Variable(Variable(var_id))) => {
            //         if let Some(stream) = ports.get(var_id.as_str()) {
            //             let updated = left.apply(stream, PartialEq::eq);
            //             if updated.0.is_empty() {
            //                 return Err(());
            //             }

            //             ports.insert(var_id.as_str(), updated.0);
            //             program.remove(idx);
            //             Ok(())
            //         } else {
            //             Err(())
            //         }
            //     }
            //     (Value::Variable(Variable(var_id)), Value::Literal(right)) => {
            //         if let Some(stream) = ports.get(var_id.as_str()) {
            //             let updated = left.apply(stream, PartialEq::eq);
            //             if updated.0.is_empty() {
            //                 return Err(());
            //             }

            //             ports.insert(var_id.as_str(), updated.0);
            //             program.remove(idx);
            //             Ok(())
            //         } else {
            //             Err(())
            //         }
            //     }
            //     // (Value::Variable(left), Value::Literal(right)) => {
            //     //     if let Some(stream) = ports.get(left.as_str()) {
            //     //         let updated = stream.apply(&Ipld::String(right), &equal);
            //     //         ports.insert(left.as_str(), updated.0);
            //     //     }

            //     //     program.remove(name);
            //     // }
            //     // (Value::Variable(left), Value::Variable(right)) => {
            //     //     if let Some(stream) = ports.get(left.as_str()) {
            //     //         let updated = stream.apply(&Ipld::String(right.clone()), &equal);
            //     //         ports.insert(left.as_str(), updated.0);
            //     //         // FIXME UPDATE BOTH
            //     //     }

            //     //     program.remove(name);
            //     // }
            //     _ => todo!(),
            // },
            // _ => todo!(),
        })
        .map(|_| &machine)
}

// [".[]", "$", ?x]
// [".[]", "$", ?y]
//
// these will run to exhaustion?
// [".bar", "?x", "?y"]
// [".baz", "?y", "?x"]
