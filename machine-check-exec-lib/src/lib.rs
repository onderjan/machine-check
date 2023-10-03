mod model_check;
mod precision;
mod space;

use std::collections::VecDeque;

use mck::{
    mark::{Join, MarkMachine, MarkState},
    AbstractMachine, MarkBitvector,
};
use thiserror::Error;

use self::{precision::Precision, space::Space};

use mck::FieldManipulate;

#[derive(Error, Debug)]
pub enum Error {
    #[error("incomplete verification")]
    Incomplete(Culprit),
    #[error("field '{0}' of bit type not found")]
    FieldNotFound(String),
}

#[derive(Debug)]
pub struct Culprit {
    pub path: VecDeque<usize>,
}

pub struct Info {
    pub num_states: usize,
    pub num_refinements: usize,
}

pub fn verify<M: MarkMachine>() -> (Result<bool, Error>, Info) {
    let mut refinery = Refinery::<M>::new();
    loop {
        let result = model_check::check_safety(&refinery.space);
        let culprit = match result {
            Ok(conclusion) => return (Ok(conclusion), refinery.info()),
            Err(error) => match error {
                Error::Incomplete(culprit) => culprit,
                _ => return (Err(error), refinery.info()),
            },
        };
        if !refinery.refine(&culprit) {
            return (Err(Error::Incomplete(culprit)), refinery.info());
        }
    }
}

struct Refinery<M: MarkMachine> {
    precision: Precision<M>,
    space: Space<M::Abstract>,
    num_refinements: usize,
}

impl<M: MarkMachine> Refinery<M> {
    fn new() -> Self {
        let mut refinery = Refinery {
            precision: Precision::new(),
            space: Space::new(),
            num_refinements: 0,
        };
        // generate first space
        refinery.regenerate_init();
        refinery
    }

    fn refine(&mut self, culprit: &Culprit) -> bool {
        let safe_str = "safe";

        self.num_refinements += 1;
        // compute marking
        let mut current_state_mark = M::State::new_unmarked();
        let mark_bit = current_state_mark.get_mut(safe_str).unwrap();
        *mark_bit = MarkBitvector::new_marked();

        // try increasing precision of the state preceding current mark
        let previous_state_iter = culprit.path.iter().cloned().rev().skip(1);
        let current_state_iter = culprit.path.iter().cloned().rev();
        let iter = previous_state_iter.zip(current_state_iter);

        for (previous_state_index, current_state_index) in iter {
            let previous_state = self.space.get_state_by_index(previous_state_index);

            let input = &self
                .space
                .get_representative_step_input(previous_state_index, current_state_index);

            // step using the previous state as input
            let (new_state_mark, input_mark) =
                <M as MarkMachine>::next((previous_state, input), current_state_mark);
            let previous_state_precision = self.precision.get_for_state_mut(previous_state_index);

            let mut joined_precision: M::Input = previous_state_precision.clone();
            Join::apply_join(&mut joined_precision, input_mark);
            if previous_state_precision != &joined_precision {
                *previous_state_precision = joined_precision;
                // regenerate step from the state
                let mut queue = VecDeque::new();
                queue.push_back(previous_state_index);
                self.regenerate_step(queue);
                return true;
            }

            current_state_mark = new_state_mark;
        }

        let initial_state = culprit.path.front().unwrap();

        let init_input = self.space.get_representative_init_input(*initial_state);

        // increasing state precision failed, try increasing init precision
        let (input_mark,) = <M as MarkMachine>::init((init_input,), current_state_mark);

        let init_precision = self.precision.get_init_mut();
        let mut joined_precision: M::Input = init_precision.clone();
        Join::apply_join(&mut joined_precision, input_mark);
        if *init_precision != joined_precision {
            *init_precision = joined_precision;
            // regenerate init
            self.regenerate_init();
            return true;
        }

        // incomplete
        false
    }

    pub fn regenerate_init(&mut self) {
        // remove initial states
        self.space.remove_initial_states();

        // regenerate them using init function with init precision
        // remember the states that were actually added
        let mut added_states_queue = VecDeque::new();

        let initial_precision = self.precision.get_init();
        for input in M::input_precision_iter(initial_precision) {
            let (initial_state_id, added) = self
                .space
                .add_initial_state(M::Abstract::init(&input), &input);
            if added {
                added_states_queue.push_back(initial_state_id);
            }
        }

        // generate every state that was added
        self.regenerate_step(added_states_queue);
    }

    pub fn regenerate_step(&mut self, mut queue: VecDeque<usize>) {
        // construct state space by breadth-first search
        while let Some(current_state_index) = queue.pop_front() {
            // remove outgoing edges
            self.space.remove_outgoing_edges(current_state_index);

            // prepare state and precision
            let state = self.space.get_state_by_index(current_state_index).clone();
            let step_precision = self.precision.get_for_state(current_state_index);

            // generate direct successors
            for input in M::input_precision_iter(&step_precision) {
                let next_state = M::Abstract::next(&state, &input);

                let (next_state_index, added) =
                    self.space.add_step(current_state_index, next_state, &input);

                if added {
                    // add to queue
                    queue.push_back(next_state_index);
                }
            }
        }
    }

    pub fn info(&self) -> Info {
        Info {
            num_states: self.space.num_states(),
            num_refinements: self.num_refinements,
        }
    }
}
