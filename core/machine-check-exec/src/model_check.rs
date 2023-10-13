mod classic;
mod deduce;

use machine_check_common::{ExecError, StateId};
use mck::abstr::{Input, State};

use crate::proposition::Proposition;

use self::{classic::ClassicChecker, deduce::deduce_culprit};

use super::space::Space;

pub fn check_prop<I: Input, S: State>(
    space: &Space<I, S>,
    prop: &Proposition,
) -> Result<bool, ExecError> {
    let mut checker = ThreeValuedChecker::new(space);
    checker.check_prop(prop)
}

struct ThreeValuedChecker<'a, I: Input, S: State> {
    space: &'a Space<I, S>,
    pessimistic: ClassicChecker<'a, I, S>,
    optimistic: ClassicChecker<'a, I, S>,
}

impl<'a, I: Input, S: State> ThreeValuedChecker<'a, I, S> {
    fn new(space: &'a Space<I, S>) -> Self {
        Self {
            space,
            pessimistic: ClassicChecker::new(space, false),
            optimistic: ClassicChecker::new(space, true),
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
            (false, true) => Err(ExecError::Incomplete(deduce_culprit(self, &prop)?)),
            (true, true) => Ok(true),
            (true, false) => panic!("optimistic interpretation should hold when pessimistic does"),
        }
    }

    fn get_state_interpretation(&self, prop: &Proposition, state_index: StateId) -> Option<bool> {
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
