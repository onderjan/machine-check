use std::collections::{BTreeMap, VecDeque};

use machine_check_common::{check::Culprit, ExecError, StateId, ThreeValued};
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
            return deduce_end(checker, prop, &path);
        }
    }

    panic!("no labelling culprit found");
}

/// Deduces the ending states of the culprit, after the ones already found.
fn deduce_end<M: FullMachine>(
    checker: &ThreeValuedChecker<M>,
    prop: &Property,
    path: &VecDeque<StateId>,
) -> Result<Culprit, ExecError> {
    assert!(checker
        .get_state_labelling(prop, *path.back().unwrap())
        .is_unknown());
    match prop {
        Property::Const(_) => {
            // never ends in const
            panic!("const should never be the labelling culprit")
        }
        Property::Atomic(literal) => {
            // culprit ends here
            Ok(Culprit {
                path: path.clone(),
                atomic_property: literal.clone(),
            })
        }
        Property::Negation(inner) => {
            // propagate to inner
            deduce_end(checker, &inner.0, path)
        }
        Property::Or(BiOperator { a, b }) => {
            // the state should be unknown in p or q
            let state_index = *path.back().unwrap();
            let a_labelling = checker.get_state_labelling(a.as_ref(), state_index);
            if a_labelling.is_unknown() {
                deduce_end(checker, a, path)
            } else {
                let b_labelling = checker.get_state_labelling(b.as_ref(), state_index);
                assert!(b_labelling.is_unknown());
                deduce_end(checker, b.as_ref(), path)
            }
        }
        Property::And(BiOperator { a, b }) => {
            // the state should be unknown in p or q
            let state_index = *path.back().unwrap();
            let a_labelling = checker.get_state_labelling(a.as_ref(), state_index);
            if a_labelling.is_unknown() {
                deduce_end(checker, a, path)
            } else {
                let b_labelling = checker.get_state_labelling(b.as_ref(), state_index);
                assert!(b_labelling.is_unknown());
                deduce_end(checker, b.as_ref(), path)
            }
        }
        Property::E(prop_temp) => match prop_temp {
            TemporalOperator::X(inner) => deduce_end_ex(checker, path, inner),
            TemporalOperator::G(inner) => deduce_end_eg(checker, path, inner),
            TemporalOperator::U(inner) => deduce_end_eu(checker, path, inner),
            _ => {
                panic!(
                    "expected {:?} to have only X, G, U temporal operators",
                    prop
                );
            }
        },
        _ => {
            panic!("expected {:?} to be minimized", prop);
        }
    }
}

fn deduce_end_ex<M: FullMachine>(
    checker: &ThreeValuedChecker<M>,
    path: &VecDeque<StateId>,
    inner: &UniOperator,
) -> Result<Culprit, ExecError> {
    // lengthen by direct successor with unknown inner
    let path_back_index = *path.back().unwrap();
    for direct_successor_index in checker.space.direct_successor_iter(path_back_index.into()) {
        let direct_successor_labelling =
            checker.get_state_labelling(inner.0.as_ref(), direct_successor_index);
        if direct_successor_labelling.is_unknown() {
            // add to path
            let mut path = path.clone();
            path.push_back(direct_successor_index);
            return deduce_end(checker, &inner.0, &path);
        }
    }
    panic!("no EX culprit found")
}

fn deduce_end_eg<M: FullMachine>(
    checker: &ThreeValuedChecker<M>,
    path: &VecDeque<StateId>,
    inner: &OperatorG,
) -> Result<Culprit, ExecError> {
    // breadth-first search to find incomplete inner
    // if inner becomes false, we do not inspect the state or successors further
    let mut queue = VecDeque::new();
    let mut backtrack_map = BTreeMap::new();
    let path_back_index = *path.back().unwrap();
    queue.push_back(path_back_index);
    backtrack_map.insert(path_back_index, path_back_index);
    while let Some(state_index) = queue.pop_front() {
        let inner_labelling = checker.get_state_labelling(&inner.0, state_index);
        match inner_labelling {
            ThreeValued::True => {
                // continue down this path
                for direct_successor in checker.space.direct_successor_iter(state_index.into()) {
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

                let mut path = path.clone();
                path.append(&mut suffix);

                return deduce_end(checker, &inner.0, &path);
            }
        }
    }
    panic!("no EG culprit found");
}

fn deduce_end_eu<M: FullMachine>(
    checker: &ThreeValuedChecker<M>,
    path: &VecDeque<StateId>,
    inner: &OperatorU,
) -> Result<Culprit, ExecError> {
    // breadth-first search to find the hold or until that is incomplete
    // if the hold becomes false or until becomes true, we do not inspect the state or successors further
    let mut queue = VecDeque::new();
    let mut backtrack_map = BTreeMap::new();
    let path_back_index = *path.back().unwrap();
    queue.push_back(path_back_index);
    backtrack_map.insert(path_back_index, path_back_index);
    while let Some(state_index) = queue.pop_front() {
        let hold_labelling = checker.get_state_labelling(&inner.hold, state_index);
        let until_labelling = checker.get_state_labelling(&inner.until, state_index);
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
            for direct_successor in checker.space.direct_successor_iter(state_index.into()) {
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

        let mut path = path.clone();
        path.append(&mut suffix);

        return if hold_labelling.is_unknown() {
            deduce_end(checker, &inner.hold, &path)
        } else {
            assert!(until_labelling.is_unknown());
            deduce_end(checker, &inner.until, &path)
        };
    }
    panic!("no EU culprit found");
}
