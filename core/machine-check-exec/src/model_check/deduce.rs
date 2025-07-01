use core::panic;
use std::collections::{BTreeMap, HashSet, VecDeque};

use machine_check_common::{
    check::Culprit, property::FixedPointVariable, ExecError, StateId, ThreeValued,
};
use mck::concr::FullMachine;

use machine_check_common::property::{
    BiOperator, OperatorG, OperatorU, Property, TemporalOperator, UniOperator,
};

use super::ThreeValuedChecker;

/// Deduces the culprit of unknown three-valued model-checking result.
pub(super) fn deduce_culprit<M: FullMachine>(
    checker: &ThreeValuedChecker<M>,
    prop: &Property,
) -> Result<Culprit, ExecError> {
    // incomplete, compute culprit
    // it must start with one of the initial states
    for initial_index in checker.space.initial_iter() {
        if checker
            .get_state_labelling(prop, initial_index)
            .is_unknown()
        {
            // unknown initial state, compute culprit from it
            let mut path = VecDeque::new();
            path.push_back(initial_index);
            let mut deducer = Deducer::<M> {
                checker,
                path,
                disallowed: vec![HashSet::<StateId>::from_iter([initial_index])],
            };
            let Deduction::Culprit(culprit) = deducer.deduce_end(prop)? else {
                panic!("Deduction should give the culprit");
            };
            assert_eq!(deducer.disallowed.len(), 1);
            return Ok(culprit);
        }
    }

    unreachable!("no labelling culprit found");
}

struct Deducer<'a, M: FullMachine> {
    checker: &'a ThreeValuedChecker<'a, M>,
    path: VecDeque<StateId>,
    disallowed: Vec<HashSet<StateId>>,
}

#[derive(Debug)]
struct FixedPointDeduction {
    path: VecDeque<StateId>,
    variable: FixedPointVariable,
}

#[derive(Debug)]
enum Deduction {
    Culprit(Culprit),
    FixedPoint(FixedPointDeduction),
}

impl<M: FullMachine> Deducer<'_, M> {
    /// Deduces the ending states of the culprit, after the ones already found.
    fn deduce_end(&mut self, prop: &Property) -> Result<Deduction, ExecError> {
        //println!("Space: {:?}", self.checker.space);
        //println!("Deducing end for property {}", prop);
        assert!(self
            .checker
            .get_state_labelling(prop, *self.path.back().unwrap())
            .is_unknown());
        match prop {
            Property::Const(_) => {
                // never ends in const
                panic!("const should never be the labelling culprit")
            }
            Property::Atomic(literal) => {
                // culprit ends here
                Ok(Deduction::Culprit(Culprit {
                    path: self.path.clone(),
                    atomic_property: literal.clone(),
                }))
            }
            Property::Negation(inner) => {
                // propagate to inner
                self.deduce_end(&inner.0)
            }
            Property::And(BiOperator { a, b }) | Property::Or(BiOperator { a, b }) => {
                // the state should be unknown in p or q
                let state_index = *self.path.back().unwrap();
                let a_labelling = self.checker.get_state_labelling(a.as_ref(), state_index);
                let a_deduction = if a_labelling.is_unknown() {
                    let a_deduction = self.deduce_end(a)?;
                    if matches!(a_deduction, Deduction::Culprit(_)) {
                        return Ok(a_deduction);
                    }
                    Some(a_deduction)
                } else {
                    None
                };
                let b_labelling = self.checker.get_state_labelling(b.as_ref(), state_index);
                assert!(b_labelling.is_unknown());
                let b_deduction = self.deduce_end(b.as_ref())?;
                if matches!(b_deduction, Deduction::Culprit(_)) {
                    return Ok(b_deduction);
                }
                // prefer the left deduction over the right one
                Ok(a_deduction.unwrap_or(b_deduction))
            }
            Property::E(prop_temp) => match prop_temp {
                TemporalOperator::X(inner) => {
                    let path_back_index = *self.path.back().unwrap();
                    let reason = self
                        .checker
                        .get_state_labelling_reason(prop, path_back_index)
                        .expect("Culprit state should have a labelling reason");
                    //println!("EX reason: {:?}", reason);
                    self.deduce_end_next(inner, reason)
                }
                TemporalOperator::G(inner) => self.deduce_end_eg(inner),
                TemporalOperator::U(inner) => self.deduce_end_eu(inner),
                _ => {
                    panic!(
                        "expected {:?} to have only X, G, U temporal operators",
                        prop
                    );
                }
            },
            /*Property::A(prop_temp) => match prop_temp {
                TemporalOperator::X(inner) => self.deduce_end_next(inner),
                _ => {
                    panic!("expected {:?} to have only X temporal operator", prop);
                }
            },*/
            Property::LeastFixedPoint(operator) | Property::GreatestFixedPoint(operator) => {
                self.disallowed.push(HashSet::new());
                let result = loop {
                    let deduction = self.deduce_end(&operator.inner)?;
                    match deduction {
                        Deduction::Culprit(_) => break Ok(deduction),
                        Deduction::FixedPoint(deduction) => {
                            if deduction.variable != operator.variable {
                                // not our variable, break
                                break Ok(Deduction::FixedPoint(deduction));
                            }
                            // our variable, update path and loop
                            self.path = deduction.path;
                        }
                    }
                };
                self.disallowed.pop();
                result
            }
            Property::FixedPointVariable(variable) => {
                // return fixed-point deduction
                Ok(Deduction::FixedPoint(FixedPointDeduction {
                    path: self.path.clone(),
                    variable: variable.clone(),
                }))
            }
            _ => {
                panic!("expected {:?} to be minimized", prop);
            }
        }
    }

    fn deduce_end_next(
        &mut self,
        inner: &UniOperator,
        reason: StateId,
    ) -> Result<Deduction, ExecError> {
        //println!("Deducing end for path: {:?}", self.path);
        // lengthen by direct successor with unknown inner
        let path_back_index = *self.path.back().unwrap();
        let disallowed_states = self
            .disallowed
            .last_mut()
            .expect("There should be a disallowed states scope");

        for direct_successor_index in self
            .checker
            .space
            .direct_successor_iter(path_back_index.into())
        {
            /*println!(
                "Considering {} -> {}",
                path_back_index, direct_successor_index
            );*/
            /*if disallowed_states.contains(&direct_successor_index) {
                println!("Disallowed");
                continue;
            }*/

            if direct_successor_index != reason {
                //println!("Not the reason");
                continue;
            }

            let direct_successor_labelling = self
                .checker
                .get_state_labelling(inner.0.as_ref(), direct_successor_index);

            if direct_successor_labelling.is_unknown() {
                // add to path
                //println!("Unknown, adding to path");
                self.path.push_back(direct_successor_index);
                disallowed_states.insert(direct_successor_index);
                return self.deduce_end(&inner.0);
            } else {
                //println!("Not unknown");
            }
        }
        panic!("no next state culprit found")
    }

    fn deduce_end_eg(&mut self, inner: &OperatorG) -> Result<Deduction, ExecError> {
        // breadth-first search to find incomplete inner
        // if inner becomes false, we do not inspect the state or successors further
        let mut queue = VecDeque::new();
        let mut backtrack_map = BTreeMap::new();
        let path_back_index = *self.path.back().unwrap();
        queue.push_back(path_back_index);
        backtrack_map.insert(path_back_index, path_back_index);
        while let Some(state_index) = queue.pop_front() {
            let inner_labelling = self.checker.get_state_labelling(&inner.0, state_index);
            match inner_labelling {
                ThreeValued::True => {
                    // continue down this path
                    for direct_successor in
                        self.checker.space.direct_successor_iter(state_index.into())
                    {
                        backtrack_map.entry(direct_successor).or_insert_with(|| {
                            queue.push_back(direct_successor);
                            state_index
                        });
                    }
                }
                ThreeValued::False => {
                    // do not continue down this path, nothing can change that EG definitely does not hold here
                }
                ThreeValued::Unknown => {
                    // reconstruct the path to the state
                    let mut suffix = VecDeque::new();
                    let mut backtrack_state_index = state_index;
                    loop {
                        let predecessor_state_index =
                            *backtrack_map.get(&backtrack_state_index).unwrap();
                        if predecessor_state_index == backtrack_state_index {
                            // we are already at the start index
                            break;
                        }

                        suffix.push_front(backtrack_state_index);
                        backtrack_state_index = predecessor_state_index;
                    }

                    self.path.append(&mut suffix);

                    return self.deduce_end(&inner.0);
                }
            }
        }
        panic!("no EG culprit found");
    }

    fn deduce_end_eu(&mut self, inner: &OperatorU) -> Result<Deduction, ExecError> {
        // breadth-first search to find the hold or until that is incomplete
        // if the hold becomes false or until becomes true, we do not inspect the state or successors further
        let mut queue = VecDeque::new();
        let mut backtrack_map = BTreeMap::new();
        let path_back_index = *self.path.back().unwrap();
        queue.push_back(path_back_index);
        backtrack_map.insert(path_back_index, path_back_index);
        while let Some(state_index) = queue.pop_front() {
            let hold_labelling = self.checker.get_state_labelling(&inner.hold, state_index);
            let until_labelling = self.checker.get_state_labelling(&inner.until, state_index);
            if hold_labelling.is_false() {
                // hold is false, so this state must have a known labelling that resolves this path
                // so it is pointless to continue here
                continue;
            }
            if until_labelling.is_true() {
                // until is true, so this state must have a known labelling that resolves this path
                // so it is pointless to continue here
                continue;
            }
            if hold_labelling.is_known() && until_labelling.is_known() {
                // everything is known about this state, so it is not the culprit, continue down the path
                for direct_successor in self.checker.space.direct_successor_iter(state_index.into())
                {
                    backtrack_map.entry(direct_successor).or_insert_with(|| {
                        queue.push_back(direct_successor);
                        state_index
                    });
                }
                continue;
            }
            // reconstruct the path to the state
            let mut suffix = VecDeque::new();
            let mut backtrack_state_index = state_index;
            loop {
                let predecessor_state_index = *backtrack_map.get(&backtrack_state_index).unwrap();
                if predecessor_state_index == backtrack_state_index {
                    // we are already at the start index
                    break;
                }

                suffix.push_front(backtrack_state_index);
                backtrack_state_index = predecessor_state_index;
            }

            self.path.append(&mut suffix);

            return if hold_labelling.is_unknown() {
                self.deduce_end(&inner.hold)
            } else {
                assert!(until_labelling.is_unknown());
                self.deduce_end(&inner.until)
            };
        }
        panic!("no EU culprit found");
    }
}
