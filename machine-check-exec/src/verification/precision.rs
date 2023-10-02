use std::collections::HashMap;

use mck::mark::MarkInput;
use mck::mark::MarkMachine;

pub struct Precision<M: MarkMachine> {
    init: M::MarkInput,
    step: HashMap<usize, M::MarkInput>,
}

impl<MM: MarkMachine> Precision<MM> {
    pub fn new() -> Self {
        Precision {
            init: MM::MarkInput::new_unmarked(),
            step: HashMap::new(),
        }
    }

    pub fn get_init(&self) -> &MM::MarkInput {
        &self.init
    }

    pub fn get_init_mut(&mut self) -> &mut MM::MarkInput {
        &mut self.init
    }

    pub fn get_for_state(&self, state_index: usize) -> MM::MarkInput {
        let result = self.step.get(&state_index);
        match result {
            Some(result) => result.clone(),
            None => MM::MarkInput::new_unmarked(),
        }
    }

    pub fn get_for_state_mut(&mut self, state_index: usize) -> &mut MM::MarkInput {
        self.step
            .entry(state_index)
            .or_insert_with(MM::MarkInput::new_unmarked)
    }
}
