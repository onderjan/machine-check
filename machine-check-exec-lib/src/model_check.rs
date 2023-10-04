use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use mck::AbstractMachine;

use super::{space::Space, Culprit, Error};

pub fn check_safety<AM: AbstractMachine>(space: &Space<AM>) -> Result<bool, Error> {
    // check AG[safe]
    // no complementary literal
    // in two-valued checking, transform to !E[true U !safe]
    let prop = Proposition::Negation(Box::new(Proposition::EU(PropositionEU {
        hold: Box::new(Proposition::Const(true)),
        until: Box::new(Proposition::Negation(Box::new(Proposition::Literal(
            Literal {
                complementary: false,
                name: String::from("safe"),
            },
        )))),
    })));

    let mut checker = ThreeValuedChecker::new(space);
    checker.check_prop(&prop)

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

    fn check_prop(&mut self, prop: &Proposition) -> Result<bool, Error> {
        // compute optimistic and pessimistic interpretation
        let pessimistic_interpretation = self.pessimistic.compute_interpretation(prop)?;
        let optimistic_interpretation = self.optimistic.compute_interpretation(prop)?;

        /*println!(
            "Pessimistic: {}, optimistic: {}",
            pessimistic_interpretation, optimistic_interpretation
        );*/

        match (pessimistic_interpretation, optimistic_interpretation) {
            (false, false) => Ok(false),
            (false, true) => Err(Error::Incomplete(
                self.compute_interpretation_culprit(prop)?,
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
                return self.compute_labeling_culprit(prop, &path);
            }
        }

        panic!("no interpretation culprit found");
    }

    fn compute_labeling_culprit(
        &self,
        prop: &Proposition,
        path: &VecDeque<usize>,
    ) -> Result<Culprit, Error> {
        match prop {
            Proposition::Const(_) => {
                // never ends in const
                panic!("should never consider const as labeling culprit")
            }
            Proposition::Literal(literal) => {
                assert!(
                    self.get_interpretation(prop, *path.back().unwrap())
                        .is_none(),
                    "literal labeling culprit should be unknown"
                );
                // culprit ends here
                Ok(Culprit {
                    path: path.clone(),
                    name: literal.name.clone(),
                })
            }
            Proposition::Negation(inner) => {
                // propagate to inner
                self.compute_labeling_culprit(inner, path)
            }
            Proposition::EU(eu) => {
                // breadth-first search to find the hold or until that is incomplete
                // if the hold becomes false or until becomes true, we do not inspect the state or successors further
                let mut queue = VecDeque::new();
                let mut backtrack_map = BTreeMap::new();
                let start_index = *path.back().unwrap();
                queue.push_back(start_index);
                backtrack_map.insert(start_index, start_index);
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
                        self.compute_labeling_culprit(&eu.hold, &path)
                    } else {
                        self.compute_labeling_culprit(&eu.until, &path)
                    };
                }
                panic!("no EU culprit found");
            }
        }
    }

    fn get_interpretation(&self, prop: &Proposition, state_index: usize) -> Option<bool> {
        let pessimistic_interpretation = self.pessimistic.get_labeling(prop).contains(&state_index);
        let optimistic_interpretation = self.optimistic.get_labeling(prop).contains(&state_index);
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
struct PropositionEU {
    hold: Box<Proposition>,
    until: Box<Proposition>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum Proposition {
    Const(bool),
    Literal(Literal),
    Negation(Box<Proposition>),
    EU(PropositionEU),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Index(usize);

pub struct BooleanChecker<'a, AM: AbstractMachine> {
    space: &'a Space<AM>,
    optimistic: bool,
    labeling_map: HashMap<Proposition, BTreeSet<usize>>,
}

impl<'a, AM: AbstractMachine> BooleanChecker<'a, AM> {
    fn new(space: &'a Space<AM>, optimistic: bool) -> Self {
        BooleanChecker {
            space,
            optimistic,
            labeling_map: HashMap::new(),
        }
    }

    fn compute_eu_labeling(&mut self, prop: &PropositionEU) -> Result<BTreeSet<usize>, Error> {
        // worklist-based labeling procedure CheckEU from Model Checking 1999 by Clarke et al.

        self.compute_labeling(&prop.hold)?;
        self.compute_labeling(&prop.until)?;

        let prop = prop.clone();

        let hold_labeling = self.get_labeling(&prop.hold);

        // the working set holds all states labeled "until" at the start
        let mut working = self.get_labeling(&prop.until).clone();
        // make all states in working set labelled EU(f,g)
        let mut eu_labeling = BTreeSet::new();

        // choose and process states from working set until empty
        while let Some(state_index) = working.pop_first() {
            // for every parent of the chosen state which is labeled (f) but not EU(f,g) yet,
            // label it EU(f,g) and add to the working set
            for parent in self.space.parents_iter(state_index) {
                if hold_labeling.contains(&parent) {
                    let inserted = eu_labeling.insert(parent);
                    if inserted {
                        working.insert(parent);
                    }
                }
            }
        }

        Ok(eu_labeling)
    }

    fn compute_labeling(&mut self, prop: &Proposition) -> Result<(), Error> {
        if self.labeling_map.contains_key(prop) {
            // already contained
            return Ok(());
        }

        let computed_labeling = match prop {
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
                let full_labeling = BTreeSet::from_iter(self.space.index_iter());
                self.compute_labeling(inner)?;
                let inner_labeling = self.get_labeling(inner);
                full_labeling.difference(inner_labeling).cloned().collect()
            }
            Proposition::EU(eu) => self.compute_eu_labeling(eu)?,
        };

        /*println!(
            "({}) Computed labeling of {:?}: {:?}",
            self.optimistic, prop, computed_labeling
        );*/

        // insert the labeling to labeling map for future reference
        self.labeling_map.insert(prop.clone(), computed_labeling);

        Ok(())
    }

    fn get_labeling(&self, prop: &Proposition) -> &BTreeSet<usize> {
        self.labeling_map
            .get(prop)
            .expect("labeling should be present")
    }

    fn compute_interpretation(&mut self, prop: &Proposition) -> Result<bool, Error> {
        self.compute_labeling(prop)?;
        let labeling = self.get_labeling(prop);
        // conventionally, the property must hold in all initial states
        for initial_index in self.space.initial_index_iter() {
            if !labeling.contains(&initial_index) {
                //println!("({}) false", self.optimistic);
                return Ok(false);
            }
        }
        //println!("({}) true", self.optimistic);
        Ok(true)
    }
}
