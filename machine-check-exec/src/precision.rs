use std::collections::HashMap;

use mck::mark::MarkInput;
use mck::mark::MarkMachine;
use mck::mark::MarkState;

pub struct Precision<M: MarkMachine> {
    init: M::Input,
    init_decay: M::State,
    step: HashMap<usize, M::Input>,
    step_decay: HashMap<usize, M::State>,
}

impl<M: MarkMachine> Precision<M> {
    pub fn new() -> Self {
        Precision {
            init: M::Input::new_unmarked(),
            init_decay: M::State::new_unmarked(),
            step: HashMap::new(),
            step_decay: HashMap::new(),
        }
    }

    pub fn get_init(&self) -> &M::Input {
        &self.init
    }

    pub fn init_decay(&self) -> &M::State {
        &self.init_decay
    }

    pub fn get_step(&self, state_index: usize) -> M::Input {
        let result = self.step.get(&state_index);
        match result {
            Some(result) => result.clone(),
            None => M::Input::new_unmarked(),
        }
    }

    pub fn precision_mut(&mut self, state_index: Option<&usize>) -> &mut M::Input {
        if let Some(state_index) = state_index {
            self.step
                .entry(*state_index)
                .or_insert_with(M::Input::new_unmarked)
        } else {
            &mut self.init
        }
    }

    pub fn step_decay(&self, from_state_index: usize) -> M::State {
        match self.step_decay.get(&from_state_index) {
            Some(result) => result.clone(),
            None => M::State::new_unmarked(),
        }
    }

    pub fn decay_mut(&mut self, state_index: Option<&usize>) -> &mut M::State {
        if let Some(state_index) = state_index {
            self.step_decay
                .entry(*state_index)
                .or_insert_with(M::State::new_unmarked)
        } else {
            &mut self.init_decay
        }
    }

    pub fn retain_indices<F>(&mut self, predicate: F)
    where
        F: Fn(usize) -> bool,
    {
        self.step.retain(|k, _| predicate(*k));
    }
}
