use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use mck::AbstractMachine;

use super::{space::Space, Culprit, Error};

pub fn check_safety<AM: AbstractMachine>(space: &Space<AM>) -> Result<bool, Error> {
    // check AG[safe]
    // no complementary literal
    // in two-valued checking, transform to !E[true U !safe]
    let prop = Proposition::Negation(Box::new(Proposition::EU(PropositionU {
        hold: Box::new(Proposition::Const(true)),
        until: Box::new(Proposition::Negation(Box::new(Proposition::Literal(
            Literal {
                complementary: false,
                name: String::from("safe"),
            },
        )))),
    })));

    let mut checker = ThreeValuedChecker::new(space);
    checker.check_prop(prop)

    /*
    // check AG[!bad]
    // bfs from initial states
    let mut open = VecDeque::<usize>::new();
    let mut became_open = HashSet::<usize>::new();
    let mut backtrack_map = HashMap::<usize, usize>::new();

    open.extend(space.initial_state_indices_iter());
    became_open.extend(space.initial_state_indices_iter());

    while let Some(state_index) = open.pop_front() {
        let state = space.get_state_by_index(state_index);

        // check state
        let Some(safe) = state.get(safe_str) else {
            return Err(Error::FieldNotFound(String::from(safe_str)));
        };
        let true_bitvector = ThreeValuedBitvector::<1>::new(1);
        let false_bitvector = ThreeValuedBitvector::<1>::new(0);

        if safe == true_bitvector {
            // alright
        } else if safe == false_bitvector {
            // definitely false
            return Ok(false);
        } else {
            // unknown, put together culprit path
            let mut path = VecDeque::<usize>::new();
            path.push_front(state_index);
            let mut current_index = state_index;
            while let Some(prev_index) = backtrack_map.get(&current_index) {
                current_index = *prev_index;
                path.push_front(current_index);
            }

            return Err(Error::Incomplete(Culprit { path }));
        }

        for direct_successor_index in space.direct_successor_indices_iter(state_index) {
            let inserted = became_open.insert(direct_successor_index);
            if inserted {
                backtrack_map.insert(direct_successor_index, state_index);
                open.push_back(direct_successor_index);
            }
        }
    }
    // if no bad result was found, the result is true
    Ok(true)*/
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

    fn check_prop(&mut self, mut prop: Proposition) -> Result<bool, Error> {
        // transform to positive normal form to move negations to literals
        prop.pnf();
        // transform to existential normal form to be able to verify
        prop.enf();

        // compute optimistic and pessimistic interpretation
        let pessimistic_interpretation = self.pessimistic.compute_interpretation(&prop)?;
        let optimistic_interpretation = self.optimistic.compute_interpretation(&prop)?;

        /*println!(
            "Pessimistic: {}, optimistic: {}",
            pessimistic_interpretation, optimistic_interpretation
        );*/

        match (pessimistic_interpretation, optimistic_interpretation) {
            (false, false) => Ok(false),
            (false, true) => Err(Error::Incomplete(
                self.compute_interpretation_culprit(&prop)?,
            )),
            (true, true) => Ok(true),
            (true, false) => panic!("optimistic interpretation should hold when pessimistic does"),
        }
    }

    fn compute_interpretation_culprit(&self, prop: &Proposition) -> Result<Culprit, Error> {
        // incomplete, compute culprit
        // it must start with one of the initial states
        for initial_index in self.space.initial_index_iter() {
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
        path: &VecDeque<usize>,
    ) -> Result<Culprit, Error> {
        match prop {
            Proposition::Const(_) => {
                // never ends in const
                panic!("should never consider const as labelling culprit")
            }
            Proposition::Literal(literal) => {
                assert!(
                    self.get_interpretation(prop, *path.back().unwrap())
                        .is_none(),
                    "literal labelling culprit should be unknown"
                );
                // culprit ends here
                Ok(Culprit {
                    path: path.clone(),
                    name: literal.name.clone(),
                })
            }
            Proposition::Negation(inner) => {
                // propagate to inner
                self.compute_labelling_culprit(inner, path)
            }
            Proposition::Or(p, q) => {
                // the state should be unknown in p or q
                let state_index = *path.back().unwrap();
                let p_interpretation = self.get_interpretation(p.as_ref(), state_index);
                if p_interpretation.is_none() {
                    self.compute_labelling_culprit(p, path)
                } else {
                    let q_interpretation = self.get_interpretation(q.as_ref(), state_index);
                    assert!(q_interpretation.is_none());
                    self.compute_labelling_culprit(q.as_ref(), path)
                }
            }
            Proposition::EX(inner) => {
                // lengthen by direct successor with unknown inner
                let path_back_index = *path.back().unwrap();
                for direct_successor_index in
                    self.space.direct_successor_index_iter(path_back_index)
                {
                    let direct_successor_interpretation =
                        self.get_interpretation(inner.as_ref(), direct_successor_index);
                    if direct_successor_interpretation.is_none() {
                        // add to path
                        let mut path = path.clone();
                        path.push_back(direct_successor_index);
                        return self.compute_labelling_culprit(inner, &path);
                    }
                }
                panic!("no EX culprit found")
            }
            Proposition::EG(inner) => {
                // breadth-first search to find incomplete inner
                // if inner becomes false, we do not inspect the state or successors further
                let mut queue = VecDeque::new();
                let mut backtrack_map = BTreeMap::new();
                let path_back_index = *path.back().unwrap();
                queue.push_back(path_back_index);
                backtrack_map.insert(path_back_index, path_back_index);
                while let Some(state_index) = queue.pop_front() {
                    let inner_interpretation = self.get_interpretation(inner, state_index);
                    match inner_interpretation {
                        Some(true) => {
                            // continue down this path
                            for direct_successor in
                                self.space.direct_successor_index_iter(state_index)
                            {
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

                            return self.compute_labelling_culprit(inner, &path);
                        }
                    }
                }
                panic!("no EU culprit found");
            }
            Proposition::EU(eu) => {
                // breadth-first search to find the hold or until that is incomplete
                // if the hold becomes false or until becomes true, we do not inspect the state or successors further
                let mut queue = VecDeque::new();
                let mut backtrack_map = BTreeMap::new();
                let path_back_index = *path.back().unwrap();
                queue.push_back(path_back_index);
                backtrack_map.insert(path_back_index, path_back_index);
                while let Some(state_index) = queue.pop_front() {
                    let hold_interpretation = self.get_interpretation(&eu.hold, state_index);
                    let until_interpretation = self.get_interpretation(&eu.until, state_index);
                    if let Some(false) = hold_interpretation {
                        continue;
                    }
                    if let Some(true) = until_interpretation {
                        continue;
                    }
                    if hold_interpretation.is_some() && until_interpretation.is_some() {
                        // continue down the path
                        for direct_successor in self.space.direct_successor_index_iter(state_index)
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

                    return if hold_interpretation.is_none() {
                        self.compute_labelling_culprit(&eu.hold, &path)
                    } else {
                        self.compute_labelling_culprit(&eu.until, &path)
                    };
                }
                panic!("no EU culprit found");
            }
            _ => {
                panic!("expected {:?} to be minimized", prop);
            }
        }
    }

    fn get_interpretation(&self, prop: &Proposition, state_index: usize) -> Option<bool> {
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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Literal {
    complementary: bool,
    name: String,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct PropositionU {
    hold: Box<Proposition>,
    until: Box<Proposition>,
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum Proposition {
    Const(bool),
    Literal(Literal),
    Negation(Box<Proposition>),
    Or(Box<Proposition>, Box<Proposition>),
    And(Box<Proposition>, Box<Proposition>),
    EX(Box<Proposition>),
    AX(Box<Proposition>),
    AF(Box<Proposition>),
    EF(Box<Proposition>),
    EU(PropositionU),
    AU(PropositionU),
    ER(PropositionU),
    AR(PropositionU),
    EG(Box<Proposition>),
    AG(Box<Proposition>),
}

impl Proposition {
    fn pnf(&mut self) {
        self.pnf_inner(false)
    }

    fn pnf_inner(&mut self, complement: bool) {
        // propagate negations into the literals
        match self {
            Proposition::Const(value) => {
                if complement {
                    *value = !*value;
                }
            }
            Proposition::Literal(lit) => {
                if complement {
                    lit.complementary = !lit.complementary;
                }
            }
            Proposition::Negation(inner) => {
                // flip complement
                inner.pnf_inner(!complement);
                // remove negation
                *self = *inner.clone();
            }
            Proposition::Or(p, q) => {
                p.pnf_inner(complement);
                q.pnf_inner(complement);
                if complement {
                    // !(p or q) = (!p and !q)
                    // but we retain complement, so they will be flipped
                    *self = Proposition::And(p.clone(), q.clone())
                }
            }
            Proposition::And(p, q) => {
                p.pnf_inner(complement);
                q.pnf_inner(complement);
                if complement {
                    // !(p and q) = (!p or !q)
                    // but we retain complement, so they will be flipped
                    *self = Proposition::Or(p.clone(), q.clone())
                }
            }
            Proposition::EX(inner) => {
                // !EX[p] = AX[!p], we retain complement
                inner.pnf_inner(complement);
                if complement {
                    *self = Proposition::AX(inner.clone());
                }
            }
            Proposition::AX(inner) => {
                // !AX[p] = EX[!p], we retain complement
                inner.pnf_inner(complement);
                if complement {
                    *self = Proposition::EX(inner.clone());
                }
            }
            Proposition::AF(inner) => {
                // !EF[p] = AG[!p], we retain complement
                inner.pnf_inner(complement);
                if complement {
                    *self = Proposition::AG(inner.clone());
                }
            }
            Proposition::EF(inner) => {
                // !EF[p] = AG[!p], we retain complement
                inner.pnf_inner(complement);
                if complement {
                    *self = Proposition::EG(inner.clone());
                }
            }
            Proposition::EG(inner) => {
                // !EG[p] = AF[!p], we retain complement
                inner.pnf_inner(complement);
                if complement {
                    *self = Proposition::AF(inner.clone());
                }
            }
            Proposition::AG(inner) => {
                // !AG[p] = EF[!p], we retain complement
                inner.pnf_inner(complement);
                if complement {
                    *self = Proposition::EF(inner.clone());
                }
            }
            Proposition::EU(inner) => {
                // !E[p U q] = A[!p R !q], we retain complement
                inner.hold.pnf_inner(complement);
                inner.until.pnf_inner(complement);
                if complement {
                    *self = Proposition::AR(inner.clone());
                }
            }
            Proposition::AU(inner) => {
                // !A[p U q] = E[!p R !q], we retain complement
                inner.hold.pnf_inner(complement);
                inner.until.pnf_inner(complement);
                if complement {
                    *self = Proposition::ER(inner.clone());
                }
            }
            Proposition::ER(inner) => {
                // E[p R q] = !A[!p U !q], we retain complement
                inner.hold.pnf_inner(complement);
                inner.until.pnf_inner(complement);
                if complement {
                    *self = Proposition::AR(inner.clone());
                }
            }
            Proposition::AR(inner) => {
                // A[p R q] = !E[!p U !q], we retain complement
                inner.hold.pnf_inner(complement);
                inner.until.pnf_inner(complement);
                if complement {
                    *self = Proposition::ER(inner.clone());
                }
            }
        }
    }

    fn enf(&mut self) {
        match self {
            Proposition::Const(_) => return,
            Proposition::Literal(_) => return,
            Proposition::Negation(inner) => {
                inner.enf();
                return;
            }
            Proposition::Or(_, _) => (),
            Proposition::And(p, q) => {
                // p and q = !(!p or !q)
                *self = Proposition::Negation(Box::new(Proposition::Or(
                    Box::new(Proposition::Negation(Box::clone(p))),
                    Box::new(Proposition::Negation(Box::clone(q))),
                )));
            }
            Proposition::EX(inner) => {
                inner.enf();
                return;
            }
            Proposition::AX(inner) => {
                // AX[p] = !EX[!p]
                *self = Proposition::Negation(Box::new(Proposition::EX(Box::new(
                    Proposition::Negation(Box::clone(inner)),
                ))));
            }
            Proposition::AF(inner) => {
                // AF[p] = A[true U p] = !EG[!p]
                *self = Proposition::Negation(Box::new(Proposition::EG(Box::new(
                    Proposition::Negation(Box::clone(inner)),
                ))));
            }
            Proposition::EF(inner) => {
                // EF[p] = E[true U p]
                *self = Proposition::EU(PropositionU {
                    hold: Box::new(Proposition::Const(true)),
                    until: Box::clone(inner),
                });
            }
            Proposition::EG(_) => (),
            Proposition::AG(inner) => {
                // AG[p] = !EF[!p] = !E[true U !p]
                *self = Proposition::Negation(Box::new(Proposition::EU(PropositionU {
                    hold: Box::new(Proposition::Const(true)),
                    until: Box::new(Proposition::Negation(Box::clone(inner))),
                })));
            }
            Proposition::EU(inner) => {
                inner.hold.enf();
                inner.until.enf();
                return;
            }
            Proposition::AU(inner) => {
                // A[p U q] = !(E[!q U !(p or q)] or EG[!q])
                let eu_part = Proposition::EU(PropositionU {
                    hold: Box::new(Proposition::Negation(Box::clone(&inner.until))),
                    until: Box::new(Proposition::Negation(Box::new(Proposition::Or(
                        Box::clone(&inner.hold),
                        Box::clone(&inner.until),
                    )))),
                });
                let eg_part =
                    Proposition::EG(Box::new(Proposition::Negation(Box::clone(&inner.until))));
                *self = Proposition::Negation(Box::new(Proposition::Or(
                    Box::new(eu_part),
                    Box::new(eg_part),
                )));
            }
            Proposition::ER(inner) => {
                // E[p R q] = !A[!p U !q]
                let neg_hold = Proposition::Negation(inner.hold.clone());
                let neg_until = Proposition::Negation(inner.until.clone());
                *self = Proposition::Negation(Box::new(Proposition::AU(PropositionU {
                    hold: Box::new(neg_hold),
                    until: Box::new(neg_until),
                })));
            }
            Proposition::AR(inner) => {
                // A[p R q] = !E[!p U !q]
                let neg_hold = Proposition::Negation(inner.hold.clone());
                let neg_until = Proposition::Negation(inner.until.clone());
                *self = Proposition::Negation(Box::new(Proposition::EU(PropositionU {
                    hold: Box::new(neg_hold),
                    until: Box::new(neg_until),
                })));
            }
        }
        // minimize the new expression
        self.enf();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Index(usize);

pub struct BooleanChecker<'a, AM: AbstractMachine> {
    space: &'a Space<AM>,
    optimistic: bool,
    labelling_map: HashMap<Proposition, BTreeSet<usize>>,
}

impl<'a, AM: AbstractMachine> BooleanChecker<'a, AM> {
    fn new(space: &'a Space<AM>, optimistic: bool) -> Self {
        BooleanChecker {
            space,
            optimistic,
            labelling_map: HashMap::new(),
        }
    }

    fn compute_ex_labelling(&mut self, inner: &Proposition) -> Result<BTreeSet<usize>, Error> {
        self.compute_labelling(inner)?;
        let inner_labelling = self.get_labelling(inner);
        let mut result = BTreeSet::new();
        // for each state labelled by p, mark the preceding states EX[p]
        for state_index in inner_labelling {
            for direct_predecessor_index in self.space.direct_predecessor_index_iter(*state_index) {
                result.insert(direct_predecessor_index);
            }
        }
        Ok(result)
    }

    fn compute_eg_labelling(&mut self, inner: &Proposition) -> Result<BTreeSet<usize>, Error> {
        // Boolean SCC-based labelling procedure CheckEG from Model Checking 1999 by Clarke et al.

        // compute inner labelling
        self.compute_labelling(inner)?;
        let inner_labelling = self.get_labelling(inner);

        // compute states of nontrivial strongly connected components of labelled and insert them into working set
        let mut working_set = self.space.labelled_nontrivial_scc_indices(inner_labelling);

        // make all states in working set labelled EG(f)
        let mut eg_labelling = working_set.clone();

        // choose and process states from working set until empty
        while let Some(state_index) = working_set.pop_first() {
            // for every directed predecessor of the chosen state which is labelled (f) but not EG(f) yet,
            // label it EG f and add to the working set
            for directed_predecessor_index in self.space.direct_predecessor_index_iter(state_index)
            {
                if inner_labelling.contains(&directed_predecessor_index) {
                    let inserted = eg_labelling.insert(directed_predecessor_index);
                    if inserted {
                        working_set.insert(directed_predecessor_index);
                    }
                }
            }
        }

        // return states labelled EG(f)
        Ok(eg_labelling)
    }

    fn compute_eu_labelling(&mut self, prop: &PropositionU) -> Result<BTreeSet<usize>, Error> {
        // worklist-based labelling procedure CheckEU from Model Checking 1999 by Clarke et al.

        self.compute_labelling(&prop.hold)?;
        self.compute_labelling(&prop.until)?;

        let prop = prop.clone();

        let hold_labelling = self.get_labelling(&prop.hold);

        // the working set holds all states labeled "until" at the start
        let mut working = self.get_labelling(&prop.until).clone();
        // make all states in working set labelled EU(f,g)
        let mut eu_labelling = BTreeSet::new();

        // choose and process states from working set until empty
        while let Some(state_index) = working.pop_first() {
            // for every parent of the chosen state which is labeled (f) but not EU(f,g) yet,
            // label it EU(f,g) and add to the working set
            for parent in self.space.parents_iter(state_index) {
                if hold_labelling.contains(&parent) {
                    let inserted = eu_labelling.insert(parent);
                    if inserted {
                        working.insert(parent);
                    }
                }
            }
        }

        Ok(eu_labelling)
    }

    fn compute_labelling(&mut self, prop: &Proposition) -> Result<(), Error> {
        if self.labelling_map.contains_key(prop) {
            // already contained
            return Ok(());
        }

        let computed_labelling = match prop {
            Proposition::Const(c) => {
                if *c {
                    // holds in all state indices
                    BTreeSet::from_iter(self.space.index_iter())
                } else {
                    // holds nowhere
                    BTreeSet::new()
                }
            }
            Proposition::Literal(literal) => {
                // get from space
                let labelled: Result<BTreeSet<usize>, ()> = self
                    .space
                    .labelled_index_iter(&literal.name, literal.complementary, self.optimistic)
                    .collect();
                match labelled {
                    Ok(labelled) => labelled,
                    Err(_) => return Err(Error::FieldNotFound(literal.name.clone())),
                }
            }
            Proposition::Negation(inner) => {
                // complement
                let full_labelling = BTreeSet::from_iter(self.space.index_iter());
                self.compute_labelling(inner)?;
                let inner_labelling = self.get_labelling(inner);
                full_labelling
                    .difference(inner_labelling)
                    .cloned()
                    .collect()
            }
            Proposition::Or(p, q) => {
                self.compute_labelling(p)?;
                self.compute_labelling(q)?;
                let p_labelling = self.get_labelling(p);
                let q_labelling = self.get_labelling(q);
                p_labelling.union(q_labelling).cloned().collect()
            }
            Proposition::EX(inner) => self.compute_ex_labelling(inner)?,
            Proposition::EU(eu) => self.compute_eu_labelling(eu)?,
            Proposition::EG(inner) => self.compute_eg_labelling(inner)?,
            _ => panic!("expected {:?} to be minimized", prop),
        };

        /*println!(
            "({}) Computed labelling of {:?}: {:?}",
            self.optimistic, prop, computed_labelling
        );*/

        // insert the labelling to labelling map for future reference
        self.labelling_map.insert(prop.clone(), computed_labelling);

        Ok(())
    }

    fn get_labelling(&self, prop: &Proposition) -> &BTreeSet<usize> {
        self.labelling_map
            .get(prop)
            .expect("labelling should be present")
    }

    fn compute_interpretation(&mut self, prop: &Proposition) -> Result<bool, Error> {
        self.compute_labelling(prop)?;
        let labelling = self.get_labelling(prop);
        // conventionally, the property must hold in all initial states
        for initial_index in self.space.initial_index_iter() {
            if !labelling.contains(&initial_index) {
                //println!("({}) false", self.optimistic);
                return Ok(false);
            }
        }
        //println!("({}) true", self.optimistic);
        Ok(true)
    }
}
