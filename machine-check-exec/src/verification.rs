mod model_check;
mod precision;
mod space;

use std::collections::VecDeque;

use mck::{mark::Join, MarkBitvector, Possibility};
use thiserror::Error;

use crate::machine::{mark, State};

use self::{model_check::Culprit, precision::Precision, space::Space};

#[derive(Error, Debug)]
pub enum Error {
    #[error("incomplete verification")]
    Incomplete,
}

pub struct Info {
    pub num_states: usize,
    pub num_refinements: usize,
}

pub fn verify() -> (Result<bool, Error>, Info) {
    let mut refinery = Refinery::new();
    loop {
        let result = model_check::check_safety(&refinery.space);
        let culprit = match result {
            Ok(conclusion) => return (Ok(conclusion), refinery.info()),
            Err(culprit) => culprit,
        };
        if let Err(err) = refinery.refine(&culprit) {
            return (Err(err), refinery.info());
        }
    }
}

pub struct Refinery {
    precision: Precision,
    space: Space,
    num_refinements: usize,
}

impl Refinery {
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

    fn refine(&mut self, culprit: &Culprit) -> Result<(), Error> {
        self.num_refinements += 1;
        // compute marking
        let mut current_state_mark: mark::State = mark::State {
            safe: MarkBitvector::new_marked(),
            ..Default::default()
        };

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
                mark::State::next((previous_state, input), current_state_mark);
            let previous_state_precision = self.precision.for_state_mut(previous_state_index);

            let mut joined_precision = previous_state_precision.clone();
            Join::apply_join(&mut joined_precision, input_mark);
            if previous_state_precision != &joined_precision {
                *previous_state_precision = joined_precision;
                // regenerate step from the state
                let mut queue = VecDeque::new();
                queue.push_back(previous_state_index);
                self.regenerate_step(queue);
                return Ok(());
            }

            current_state_mark = new_state_mark;
        }

        let initial_state = culprit.path.front().unwrap();

        let init_input = self.space.get_representative_init_input(*initial_state);

        // increasing state precision failed, try increasing init precision
        let (input_mark,) = mark::State::init((init_input,), current_state_mark);

        let init_precision = self.precision.init_mut();
        let mut joined_precision = init_precision.clone();
        Join::apply_join(&mut joined_precision, input_mark);
        if *init_precision != joined_precision {
            *init_precision = joined_precision;
            // regenerate init
            self.regenerate_init();
            return Ok(());
        }

        // incomplete
        Err(Error::Incomplete)
    }

    pub fn regenerate_init(&mut self) {
        // remove initial states
        self.space.remove_initial_states();

        // regenerate them using init function with init precision
        // remember the states that were actually added
        let mut added_states_queue = VecDeque::new();
        let mut input = Possibility::first_possibility(self.precision.init());
        loop {
            let (initial_state_id, added) =
                self.space.add_initial_state(State::init(&input), &input);
            if added {
                added_states_queue.push_back(initial_state_id);
            }

            if !Possibility::increment_possibility(self.precision.init(), &mut input) {
                break;
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
            let step_precision = self.precision.for_state(current_state_index);

            // generate direct successors
            let mut input = Possibility::first_possibility(&step_precision);
            loop {
                let next_state = state.next(&input);

                let (next_state_index, added) =
                    self.space.add_step(current_state_index, next_state, &input);

                if added {
                    // add to queue
                    queue.push_back(next_state_index);
                }

                if !Possibility::increment_possibility(&step_precision, &mut input) {
                    break;
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
