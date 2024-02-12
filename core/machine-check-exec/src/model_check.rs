mod classic;
mod deduce;

use std::collections::VecDeque;

use machine_check_common::ExecError;
use mck::concr::MachineCheckMachine;

use crate::{proposition::Proposition, space::StateId};

use self::{classic::ClassicChecker, deduce::deduce_culprit};

use super::space::Space;

pub(super) fn check_prop<M: MachineCheckMachine>(
    space: &Space<M>,
    prop: &Proposition,
) -> Result<Conclusion, ExecError> {
    let mut checker = ThreeValuedChecker::new(space);
    checker.check_prop(prop)
}

pub(super) enum Conclusion {
    Known(bool),
    Unknown(Culprit),
}

#[derive(Debug, Clone)]
pub(super) struct Culprit {
    pub path: VecDeque<StateId>,
    pub name: String,
}

struct ThreeValuedChecker<'a, M: MachineCheckMachine> {
    space: &'a Space<M>,
    pessimistic: ClassicChecker<'a, M>,
    optimistic: ClassicChecker<'a, M>,
}

impl<'a, M: MachineCheckMachine> ThreeValuedChecker<'a, M> {
    fn new(space: &'a Space<M>) -> Self {
        Self {
            space,
            pessimistic: ClassicChecker::new(space, false),
            optimistic: ClassicChecker::new(space, true),
        }
    }

    fn check_prop(&mut self, enf_prop: &Proposition) -> Result<Conclusion, ExecError> {
        // compute optimistic and pessimistic interpretation
        let pessimistic_interpretation = self.pessimistic.compute_interpretation(enf_prop)?;
        let optimistic_interpretation = self.optimistic.compute_interpretation(enf_prop)?;

        match (pessimistic_interpretation, optimistic_interpretation) {
            (false, false) => Ok(Conclusion::Known(false)),
            (false, true) => Ok(Conclusion::Unknown(deduce_culprit(self, enf_prop)?)),
            (true, true) => Ok(Conclusion::Known(true)),
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
