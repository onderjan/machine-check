use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use machine_check_common::{Culprit, ExecError, StateId};
use mck::AbstractMachine;

use crate::proposition::{Literal, PropBi, PropG, PropTemp, PropU, PropUni, Proposition};

use super::space::Space;

pub fn safety_proposition() -> Proposition {
    // check AG[safe]
    Proposition::Negation(PropUni::new(Proposition::E(PropTemp::U(PropU {
        hold: Box::new(Proposition::Const(true)),
        until: Box::new(Proposition::Negation(PropUni::new(Proposition::Literal(
            Literal::new(String::from("safe")),
        )))),
    }))))
}

pub fn check_prop<AM: AbstractMachine>(
    space: &Space<AM>,
    prop: &Proposition,
) -> Result<bool, ExecError> {
    let mut checker = ThreeValuedChecker::new(space);
    checker.check_prop(prop)
}

struct ThreeValuedChecker<'a, AM: AbstractMachine> {
    space: &'a Space<AM>,
    pessimistic: BooleanChecker<'a, AM>,
    optimistic: BooleanChecker<'a, AM>,
}

impl<'a, AM: AbstractMachine> ThreeValuedChecker<'a, AM> {
    fn new(space: &'a Space<AM>) -> Self {
        Self {
            space,
            pessimistic: BooleanChecker::new(space, false),
            optimistic: BooleanChecker::new(space, true),
        }
    }

    fn check_prop(&mut self, prop: &Proposition) -> Result<bool, ExecError> {
        // transform to positive normal form to move negations to literals
        let prop = prop.pnf();
        // transform to existential normal form to be able to verify
        let prop = prop.enf();

        // compute optimistic and pessimistic interpretation
        let pessimistic_interpretation = self.pessimistic.compute_interpretation(&prop)?;
        let optimistic_interpretation = self.optimistic.compute_interpretation(&prop)?;

        match (pessimistic_interpretation, optimistic_interpretation) {
            (false, false) => Ok(false),
            (false, true) => Err(ExecError::Incomplete(
                self.compute_interpretation_culprit(&prop)?,
            )),
            (true, true) => Ok(true),
            (true, false) => panic!("optimistic interpretation should hold when pessimistic does"),
        }
    }

    fn compute_interpretation_culprit(&self, prop: &Proposition) -> Result<Culprit, ExecError> {
        // incomplete, compute culprit
        // it must start with one of the initial states
        for initial_index in self.space.initial_iter() {
            if self.get_interpretation(prop, initial_index).is_none() {
                // unknown initial state, compute culprit from it
                let mut path = VecDeque::new();
                path.push_back(initial_index);
                return self.compute_labelling_culprit(prop, &path);
            }
        }

        panic!("no interpretation culprit found");
    }

    fn compute_labelling_culprit(
        &self,
        prop: &Proposition,
        path: &VecDeque<StateId>,
    ) -> Result<Culprit, ExecError> {
        assert!(self
            .get_interpretation(prop, *path.back().unwrap())
            .is_none());
        match prop {
            Proposition::Const(_) => {
                // never ends in const
                panic!("const should never be the labelling culprit")
            }
            Proposition::Literal(literal) => {
                // culprit ends here
                Ok(Culprit {
                    path: path.clone(),
                    name: String::from(literal.name()),
                })
            }
            Proposition::Negation(inner) => {
                // propagate to inner
                self.compute_labelling_culprit(&inner.0, path)
            }
            Proposition::Or(PropBi { a, b }) => {
                // the state should be unknown in p or q
                let state_index = *path.back().unwrap();
                let a_interpretation = self.get_interpretation(a.as_ref(), state_index);
                if a_interpretation.is_none() {
                    self.compute_labelling_culprit(a, path)
                } else {
                    let b_interpretation = self.get_interpretation(b.as_ref(), state_index);
                    assert!(b_interpretation.is_none());
                    self.compute_labelling_culprit(b.as_ref(), path)
                }
            }
            Proposition::E(prop_temp) => match prop_temp {
                PropTemp::X(inner) => self.compute_labelling_culprit_ex(path, inner),
                PropTemp::G(inner) => self.compute_labelling_culprit_eg(path, inner),
                PropTemp::U(inner) => self.compute_labelling_culprit_eu(path, inner),
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

    fn compute_labelling_culprit_ex(
        &self,
        path: &VecDeque<StateId>,
        inner: &PropUni,
    ) -> Result<Culprit, ExecError> {
        // lengthen by direct successor with unknown inner
        let path_back_index = *path.back().unwrap();
        for direct_successor_index in self.space.direct_successor_iter(path_back_index.into()) {
            let direct_successor_interpretation =
                self.get_interpretation(inner.0.as_ref(), direct_successor_index);
            if direct_successor_interpretation.is_none() {
                // add to path
                let mut path = path.clone();
                path.push_back(direct_successor_index);
                return self.compute_labelling_culprit(&inner.0, &path);
            }
        }
        panic!("no EX culprit found")
    }

    fn compute_labelling_culprit_eg(
        &self,
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
            let inner_interpretation = self.get_interpretation(&inner.0, state_index);
            match inner_interpretation {
                Some(true) => {
                    // continue down this path
                    for direct_successor in self.space.direct_successor_iter(state_index.into()) {
                        backtrack_map.entry(direct_successor).or_insert_with(|| {
                            queue.push_back(direct_successor);
                            state_index
                        });
                    }
                }
                Some(false) => {
                    // do not continue down this path, nothing can change that EG definitely does not hold here
                }
                None => {
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

                    return self.compute_labelling_culprit(&inner.0, &path);
                }
            }
        }
        panic!("no EG culprit found");
    }

    fn compute_labelling_culprit_eu(
        &self,
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
            let hold_interpretation = self.get_interpretation(&inner.hold, state_index);
            let until_interpretation = self.get_interpretation(&inner.until, state_index);
            if let Some(false) = hold_interpretation {
                continue;
            }
            if let Some(true) = until_interpretation {
                continue;
            }
            if hold_interpretation.is_some() && until_interpretation.is_some() {
                // continue down the path
                for direct_successor in self.space.direct_successor_iter(state_index.into()) {
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

            return if hold_interpretation.is_none() {
                self.compute_labelling_culprit(&inner.hold, &path)
            } else {
                assert!(until_interpretation.is_none());
                self.compute_labelling_culprit(&inner.until, &path)
            };
        }
        panic!("no EU culprit found");
    }

    fn get_interpretation(&self, prop: &Proposition, state_index: StateId) -> Option<bool> {
        let pessimistic_interpretation =
            self.pessimistic.get_labelling(prop).contains(&state_index);
        let optimistic_interpretation = self.optimistic.get_labelling(prop).contains(&state_index);
        match (pessimistic_interpretation, optimistic_interpretation) {
            (false, false) => Some(false),
            (false, true) => None,
            (true, true) => Some(true),
            (true, false) => {
                // do not panic here, intermediate result
                None
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Index(usize);

pub struct BooleanChecker<'a, AM: AbstractMachine> {
    space: &'a Space<AM>,
    optimistic: bool,
    labelling_map: HashMap<Proposition, BTreeSet<StateId>>,
}

impl<'a, AM: AbstractMachine> BooleanChecker<'a, AM> {
    fn new(space: &'a Space<AM>, optimistic: bool) -> Self {
        BooleanChecker {
            space,
            optimistic,
            labelling_map: HashMap::new(),
        }
    }

    fn compute_ex_labelling(&mut self, inner: &PropUni) -> Result<BTreeSet<StateId>, ExecError> {
        self.compute_labelling(&inner.0)?;
        let inner_labelling = self.get_labelling(&inner.0);
        let mut result = BTreeSet::new();
        // for each state labelled by p, mark the preceding states EX[p]
        for state_id in inner_labelling {
            for direct_predecessor_id in self.space.direct_predecessor_iter((*state_id).into()) {
                // ignore start node
                if let Ok(direct_predecessor_id) = StateId::try_from(direct_predecessor_id) {
                    result.insert(direct_predecessor_id);
                }
            }
        }
        Ok(result)
    }

    fn compute_eg_labelling(&mut self, inner: &PropG) -> Result<BTreeSet<StateId>, ExecError> {
        // Boolean SCC-based labelling procedure CheckEG from Model Checking 1999 by Clarke et al.

        // compute inner labelling
        self.compute_labelling(&inner.0)?;
        let inner_labelling = self.get_labelling(&inner.0);

        // compute states of nontrivial strongly connected components of labelled and insert them into working set
        let mut working_set = self.space.labelled_nontrivial_scc_indices(inner_labelling);

        // make all states in working set labelled EG(f)
        let mut eg_labelling = working_set.clone();

        // choose and process states from working set until empty
        while let Some(state_id) = working_set.pop_first() {
            // for every directed predecessor of the chosen state which is labelled (f) but not EG(f) yet,
            // label it EG f and add to the working set
            for previous_id in self.space.direct_predecessor_iter(state_id.into()) {
                // ignore start node
                if let Ok(previous_id) = StateId::try_from(previous_id) {
                    if inner_labelling.contains(&previous_id) {
                        let inserted = eg_labelling.insert(previous_id);
                        if inserted {
                            working_set.insert(previous_id);
                        }
                    }
                }
            }
        }

        // return states labelled EG(f)
        Ok(eg_labelling)
    }

    fn compute_eu_labelling(&mut self, prop: &PropU) -> Result<BTreeSet<StateId>, ExecError> {
        // worklist-based labelling procedure CheckEU from Model Checking 1999 by Clarke et al.

        self.compute_labelling(&prop.hold)?;
        self.compute_labelling(&prop.until)?;

        let prop = prop.clone();

        let hold_labelling = self.get_labelling(&prop.hold);
        let until_labelling = self.get_labelling(&prop.until);

        // the working set holds all states labeled "until" at the start
        let mut working = until_labelling.clone();
        // make all states in working set labelled EU(f,g)
        let mut eu_labelling = working.clone();

        // choose and process states from working set until empty
        while let Some(state_index) = working.pop_first() {
            // for every parent of the chosen state which is labeled (f) but not EU(f,g) yet,
            // label it EU(f,g) and add to the working set
            for previous_id in self.space.direct_predecessor_iter(state_index.into()) {
                // ignore start node
                if let Ok(previous_id) = StateId::try_from(previous_id) {
                    if hold_labelling.contains(&previous_id) {
                        let inserted = eu_labelling.insert(previous_id);
                        if inserted {
                            working.insert(previous_id);
                        }
                    }
                }
            }
        }

        Ok(eu_labelling)
    }

    fn compute_labelling(&mut self, prop: &Proposition) -> Result<(), ExecError> {
        if self.labelling_map.contains_key(prop) {
            // already contained
            return Ok(());
        }

        let computed_labelling = match prop {
            Proposition::Const(c) => {
                if *c {
                    // holds in all state indices
                    BTreeSet::from_iter(self.space.state_id_iter())
                } else {
                    // holds nowhere
                    BTreeSet::new()
                }
            }
            Proposition::Literal(literal) => {
                // get from space
                let labelled: Result<BTreeSet<_>, ()> = self
                    .space
                    .labelled_iter(literal.name(), literal.is_complementary(), self.optimistic)
                    .collect();
                match labelled {
                    Ok(labelled) => labelled,
                    Err(_) => return Err(ExecError::FieldNotFound(String::from(literal.name()))),
                }
            }
            Proposition::Negation(inner) => {
                // complement
                let full_labelling = BTreeSet::from_iter(self.space.state_id_iter());
                self.compute_labelling(&inner.0)?;
                let inner_labelling = self.get_labelling(&inner.0);
                full_labelling
                    .difference(inner_labelling)
                    .cloned()
                    .collect()
            }
            Proposition::Or(PropBi { a, b }) => {
                self.compute_labelling(a)?;
                self.compute_labelling(b)?;
                let a_labelling = self.get_labelling(a);
                let b_labelling = self.get_labelling(b);
                a_labelling.union(b_labelling).cloned().collect()
            }
            Proposition::E(prop_temp) => match prop_temp {
                PropTemp::X(inner) => self.compute_ex_labelling(inner)?,
                PropTemp::G(inner) => self.compute_eg_labelling(inner)?,
                PropTemp::U(inner) => self.compute_eu_labelling(inner)?,
                _ => {
                    panic!(
                        "expected {:?} to have only X, G, U temporal operators",
                        prop
                    );
                }
            },
            _ => panic!("expected {:?} to be minimized", prop),
        };

        // insert the labelling to labelling map for future reference
        self.labelling_map.insert(prop.clone(), computed_labelling);

        Ok(())
    }

    fn get_labelling(&self, prop: &Proposition) -> &BTreeSet<StateId> {
        self.labelling_map
            .get(prop)
            .expect("labelling should be present")
    }

    fn compute_interpretation(&mut self, prop: &Proposition) -> Result<bool, ExecError> {
        self.compute_labelling(prop)?;
        let labelling = self.get_labelling(prop);
        // conventionally, the property must hold in all initial states
        for initial_index in self.space.initial_iter() {
            if !labelling.contains(&initial_index) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
