use std::collections::HashMap;

use crate::machine::mark::Input;

pub struct Precision {
    init: Input,
    step: HashMap<usize, Input>,
}

impl Precision {
    pub fn new() -> Self {
        Precision {
            init: Input::default(),
            step: HashMap::new(),
        }
    }

    pub fn init(&self) -> &Input {
        &self.init
    }

    pub fn init_mut(&mut self) -> &mut Input {
        &mut self.init
    }

    pub fn for_state(&self, state_index: usize) -> Input {
        let result = self.step.get(&state_index);
        match result {
            Some(result) => result.clone(),
            None => Default::default(),
        }
    }

    pub fn for_state_mut(&mut self, state_index: usize) -> &mut Input {
        self.step.entry(state_index).or_insert_with(Input::default)
    }
}
