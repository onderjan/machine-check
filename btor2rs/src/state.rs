use super::{refs::Lref, refs::Rref};
use anyhow::anyhow;

#[derive(Debug, Clone)]
pub struct State {
    init: Option<Rref>,
    next: Option<Rref>,
}

impl State {
    pub(crate) fn new() -> Self {
        State {
            init: None,
            next: None,
        }
    }

    pub fn supply_init(&mut self, state: Lref, init: Rref) -> Result<(), anyhow::Error> {
        if state.sort == init.sort {
            self.init = Some(init);
            Ok(())
        } else {
            Err(anyhow!(
                "State sort {:?} is different from init sort {:?}",
                state.sort,
                init.sort
            ))
        }
    }

    pub fn supply_next(&mut self, state: Lref, next: Rref) -> Result<(), anyhow::Error> {
        if state.sort == next.sort {
            self.next = Some(next);
            Ok(())
        } else {
            Err(anyhow!(
                "State sort {:?} is different from next sort {:?}",
                state.sort,
                next.sort
            ))
        }
    }

    pub fn init(&self) -> Option<&Rref> {
        self.init.as_ref()
    }

    pub fn next(&self) -> Option<&Rref> {
        self.next.as_ref()
    }
}
