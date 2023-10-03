use std::collections::HashMap;

use mck::mark::MarkInput;
use mck::mark::MarkMachine;

pub struct Precision<M: MarkMachine> {
    init: M::Input,
    step: HashMap<usize, M::Input>,
}

impl<MM: MarkMachine> Precision<MM> {
    pub fn new() -> Self {
        Precision {
            init: MM::Input::new_unmarked(),
            step: HashMap::new(),
        }
    }

    pub fn get_init(&self) -> &MM::Input {
        &self.init
    }

    pub fn get_init_mut(&mut self) -> &mut MM::Input {
        &mut self.init
    }

    pub fn get_for_state(&self, state_index: usize) -> MM::Input {
        let result = self.step.get(&state_index);
        match result {
            Some(result) => result.clone(),
            None => MM::Input::new_unmarked(),
        }
    }

    pub fn get_for_state_mut(&mut self, state_index: usize) -> &mut MM::Input {
        self.step
            .entry(state_index)
            .or_insert_with(MM::Input::new_unmarked)
    }
}
