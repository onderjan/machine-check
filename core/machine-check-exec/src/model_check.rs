mod classic;
mod deduce;

use std::collections::VecDeque;

use log::trace;
use machine_check_common::ExecError;
use mck::concr::FullMachine;
use serde::{Deserialize, Serialize};

use crate::{
    proposition::{Literal, Proposition},
    space::StateId,
};

use self::{classic::ClassicChecker, deduce::deduce_culprit};

use super::space::Space;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedProperty(Proposition);

impl PreparedProperty {
    /// Turns the CTL proposition into a form suitable for three-valued checking.
    ///
    /// The proposition must be first converted to Positive Normal Form so that
    /// negations are turned into complementary literals, then converted to
    /// Existential Normal Form. This way, the complementary literals can be used
    /// for optimistic/pessimistic labelling while a normal ENF model-checking
    /// algorithm can be used.
    pub fn new(prop: &Proposition) -> Self {
        trace!("Original proposition: {:#?}", prop);
        // transform proposition to positive normal form to move negations to literals
        let prop = prop.pnf();
        trace!("Positive normal form: {:#?}", prop);
        // transform proposition to existential normal form to be able to verify
        let prop = prop.enf();
        trace!("Existential normal form: {:#?}", prop);
        PreparedProperty(prop)
    }
}

/// Perform three-valued model checking.
///
/// The proposition must be prepared beforehand.
pub(super) fn check_prop<M: FullMachine>(
    space: &Space<M>,
    prop: &PreparedProperty,
) -> Result<Conclusion, ExecError> {
    let mut checker = ThreeValuedChecker::new(space);
    checker.check_prop(prop)
}

/// Three-valued model-checking result.
///
/// If the result is unknown, the culprit is given.
pub(super) enum Conclusion {
    Known(bool),
    Unknown(Culprit),
}

/// The culprit of an unknown three-valued model-checking result.
///
/// Comprises of a path and a literal which is unknown in the last
/// state of the path.
#[derive(Debug, Clone)]
pub(super) struct Culprit {
    pub path: VecDeque<StateId>,
    pub literal: Literal,
}

/// Three-valued model checker.
struct ThreeValuedChecker<'a, M: FullMachine> {
    space: &'a Space<M>,
    pessimistic: ClassicChecker<'a, M>,
    optimistic: ClassicChecker<'a, M>,
}

impl<'a, M: FullMachine> ThreeValuedChecker<'a, M> {
    fn new(space: &'a Space<M>) -> Self {
        Self {
            space,
            pessimistic: ClassicChecker::new(space, false),
            optimistic: ClassicChecker::new(space, true),
        }
    }

    /// Model-checks a CTL proposition.
    ///
    /// The proposition must be prepared beforehand.
    fn check_prop(&mut self, prop: &PreparedProperty) -> Result<Conclusion, ExecError> {
        let prop = &prop.0;
        // compute optimistic and pessimistic interpretation and get the conclusion from that
        let pessimistic_interpretation = self.pessimistic.compute_interpretation(prop)?;
        let optimistic_interpretation = self.optimistic.compute_interpretation(prop)?;

        match (pessimistic_interpretation, optimistic_interpretation) {
            (false, false) => Ok(Conclusion::Known(false)),
            (false, true) => Ok(Conclusion::Unknown(deduce_culprit(self, prop)?)),
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
