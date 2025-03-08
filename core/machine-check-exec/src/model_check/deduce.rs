use std::collections::{BTreeMap, VecDeque};

use machine_check_common::{ExecError, ThreeValued};
use mck::concr::FullMachine;

use crate::{
    proposition::{PropBi, PropG, PropTemp, PropU, PropUni, Proposition},
    space::StateId,
};

use super::{Culprit, ThreeValuedChecker};

/// Deduces the culprit of unknown three-valued model-checking result.
pub(super) fn deduce_culprit<M: FullMachine>(
    checker: &ThreeValuedChecker<M>,
    prop: &Proposition,
) -> Result<Culprit, ExecError> {
    // incomplete, compute culprit
    // it must start with one of the initial states
    for initial_index in checker.space.initial_iter() {
        if checker
            .get_state_interpretation(prop, initial_index)
            .is_unknown()
        {
            // unknown initial state, compute culprit from it
            let mut path = VecDeque::new();
            path.push_back(initial_index);
            return deduce_end(checker, prop, &path);
        }
    }

    panic!("no interpretation culprit found");
}

/// Deduces the ending states of the culprit, after the ones already found.
fn deduce_end<M: FullMachine>(
    checker: &ThreeValuedChecker<M>,
    prop: &Proposition,
    path: &VecDeque<StateId>,
) -> Result<Culprit, ExecError> {
    assert!(checker
        .get_state_interpretation(prop, *path.back().unwrap())
        .is_unknown());
    match prop {
        Proposition::Const(_) => {
            // never ends in const
            panic!("const should never be the labelling culprit")
        }
        Proposition::Literal(literal) => {
            // culprit ends here
            Ok(Culprit {
                path: path.clone(),
                literal: literal.clone(),
            })
        }
        Proposition::Negation(inner) => {
            // propagate to inner
            deduce_end(checker, &inner.0, path)
        }
        Proposition::Or(PropBi { a, b }) => {
            // the state should be unknown in p or q
            let state_index = *path.back().unwrap();
            let a_interpretation = checker.get_state_interpretation(a.as_ref(), state_index);
            if a_interpretation.is_unknown() {
                deduce_end(checker, a, path)
            } else {
                let b_interpretation = checker.get_state_interpretation(b.as_ref(), state_index);
                assert!(b_interpretation.is_unknown());
                deduce_end(checker, b.as_ref(), path)
            }
        }
        Proposition::And(PropBi { a, b }) => {
            // the state should be unknown in p or q
            let state_index = *path.back().unwrap();
            let a_interpretation = checker.get_state_interpretation(a.as_ref(), state_index);
            if a_interpretation.is_unknown() {
                deduce_end(checker, a, path)
            } else {
                let b_interpretation = checker.get_state_interpretation(b.as_ref(), state_index);
                assert!(b_interpretation.is_unknown());
                deduce_end(checker, b.as_ref(), path)
            }
        }
        Proposition::E(prop_temp) => match prop_temp {
            PropTemp::X(inner) => deduce_end_ex(checker, path, inner),
            PropTemp::G(inner) => deduce_end_eg(checker, path, inner),
            PropTemp::U(inner) => deduce_end_eu(checker, path, inner),
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
    inner: &PropUni,
) -> Result<Culprit, ExecError> {
    // lengthen by direct successor with unknown inner
    let path_back_index = *path.back().unwrap();
    for direct_successor_index in checker.space.direct_successor_iter(path_back_index.into()) {
        let direct_successor_interpretation =
            checker.get_state_interpretation(inner.0.as_ref(), direct_successor_index);
        if direct_successor_interpretation.is_unknown() {
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
    inner: &PropG,
) -> Result<Culprit, ExecError> {
    // breadth-first search to find incomplete inner
    // if inner becomes false, we do not inspect the state or successors further
    let mut queue = VecDeque::new();
    let mut backtrack_map = BTreeMap::new();
    let path_back_index = *path.back().unwrap();
    queue.push_back(path_back_index);
    backtrack_map.insert(path_back_index, path_back_index);
    while let Some(state_index) = queue.pop_front() {
        let inner_interpretation = checker.get_state_interpretation(&inner.0, state_index);
        match inner_interpretation {
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
    inner: &PropU,
) -> Result<Culprit, ExecError> {
    // breadth-first search to find the hold or until that is incomplete
    // if the hold becomes false or until becomes true, we do not inspect the state or successors further
    let mut queue = VecDeque::new();
    let mut backtrack_map = BTreeMap::new();
    let path_back_index = *path.back().unwrap();
    queue.push_back(path_back_index);
    backtrack_map.insert(path_back_index, path_back_index);
    while let Some(state_index) = queue.pop_front() {
        let hold_interpretation = checker.get_state_interpretation(&inner.hold, state_index);
        let until_interpretation = checker.get_state_interpretation(&inner.until, state_index);
        if hold_interpretation.is_false() {
            // hold is false, so this state must have a known labelling that resolves this path
            // so it is pointless to continue here
            continue;
        }
        if until_interpretation.is_true() {
            // until is true, so this state must have a known labelling that resolves this path
            // so it is pointless to continue here
            continue;
        }
        if hold_interpretation.is_known() && until_interpretation.is_known() {
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

        return if hold_interpretation.is_unknown() {
            deduce_end(checker, &inner.hold, &path)
        } else {
            assert!(until_interpretation.is_unknown());
            deduce_end(checker, &inner.until, &path)
        };
    }
    panic!("no EU culprit found");
}
