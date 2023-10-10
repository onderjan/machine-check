mod classic;
mod deduce;

use machine_check_common::{ExecError, StateId};
use mck::AbstractMachine;

use crate::proposition::{Literal, PropTemp, PropU, PropUni, Proposition};

use self::{classic::ClassicChecker, deduce::deduce_culprit};

use super::space::Space;

pub fn safety_proposition() -> Proposition {
    // check AG[safe]
    Proposition::Negation(PropUni::new(Proposition::E(PropTemp::U(PropU {
        hold: Box::new(Proposition::Const(true)),
        until: Box::new(Proposition::Negation(PropUni::new(Proposition::Literal(
            Literal::new(String::from("safe")),
        )))),
    }))))
}

pub fn check_prop<AM: AbstractMachine>(
    space: &Space<AM>,
    prop: &Proposition,
) -> Result<bool, ExecError> {
    let mut checker = ThreeValuedChecker::new(space);
    checker.check_prop(prop)
}

struct ThreeValuedChecker<'a, AM: AbstractMachine> {
    space: &'a Space<AM>,
    pessimistic: ClassicChecker<'a, AM>,
    optimistic: ClassicChecker<'a, AM>,
}

impl<'a, AM: AbstractMachine> ThreeValuedChecker<'a, AM> {
    fn new(space: &'a Space<AM>) -> Self {
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Index(usize);
