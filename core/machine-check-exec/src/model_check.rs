mod classic;
mod deduce;

use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
};

use log::trace;
use machine_check_common::{ExecError, ThreeValued};
use mck::concr::FullMachine;
use serde::{Deserialize, Serialize};

use crate::{
    property::{AtomicProperty, Property},
    space::StateId,
};

use self::{classic::ClassicChecker, deduce::deduce_culprit};

use super::space::Space;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PreparedProperty {
    original: Property,
    prepared: Property,
}

impl PreparedProperty {
    /// Turns the CTL proposition into a form suitable for three-valued checking.
    ///
    /// The proposition must be first converted to Positive Normal Form so that
    /// negations are turned into complementary literals, then converted to
    /// Existential Normal Form. This way, the complementary literals can be used
    /// for optimistic/pessimistic labelling while a normal ENF model-checking
    /// algorithm can be used.
    pub fn new(original_prop: Property) -> Self {
        trace!("Original proposition: {:#?}", original_prop);
        // transform proposition to positive normal form to move negations to literals
        let prop = original_prop.pnf();
        trace!("Positive normal form: {:#?}", prop);
        // transform proposition to existential normal form to be able to verify
        let prop = prop.enf();
        trace!("Existential normal form: {:#?}", prop);
        PreparedProperty {
            original: original_prop,
            prepared: prop,
        }
    }

    pub fn original(&self) -> &Property {
        &self.original
    }
}

impl Display for PreparedProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // the prepared property is just a reformulation of the original
        write!(f, "{}", self.original)
    }
}

/// Perform three-valued model checking.
///
/// The proposition must be prepared beforehand.
pub(super) fn check_property<M: FullMachine>(
    space: &Space<M>,
    prop: &PreparedProperty,
) -> Result<Conclusion, ExecError> {
    let mut checker = ThreeValuedChecker::new(space);
    checker.check_property(prop)
}

pub(super) fn compute_property_labelling<M: FullMachine>(
    space: &Space<M>,
    property: &PreparedProperty,
) -> Result<HashMap<StateId, ThreeValued>, ExecError> {
    let mut checker = ThreeValuedChecker::new(space);
    checker.compute_property_labelling(property)
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
    pub literal: AtomicProperty,
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
    fn check_property(&mut self, property: &PreparedProperty) -> Result<Conclusion, ExecError> {
        let prop = &property.prepared;
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

    pub fn compute_property_labelling(
        &mut self,
        property: &PreparedProperty,
    ) -> Result<HashMap<StateId, ThreeValued>, ExecError> {
        let prop = &property.prepared;
        // compute the optimistic and pessimistic interpretation labellings
        let pessimistic_labelling = self.pessimistic.compute_and_get_labelling(prop)?;
        let optimistic_labelling = self.optimistic.compute_and_get_labelling(prop)?;

        let mut result = HashMap::new();

        for state_id in self.space.state_id_iter() {
            let labelling = match (
                pessimistic_labelling.contains(&state_id),
                optimistic_labelling.contains(&state_id),
            ) {
                (false, false) => ThreeValued::False,
                (false, true) => ThreeValued::Unknown,
                (true, true) => ThreeValued::True,
                (true, false) => {
                    // panic here as this is not an intermediate proposition
                    panic!("optimistic interpretation should hold when pessimistic does")
                }
            };
            result.insert(state_id, labelling);
        }

        Ok(result)
    }

    fn get_state_labelling(&self, prop: &Property, state_index: StateId) -> ThreeValued {
        let pessimistic_labelling = self.pessimistic.get_labelling(prop).contains(&state_index);
        let optimistic_labelling = self.optimistic.get_labelling(prop).contains(&state_index);
        match (pessimistic_labelling, optimistic_labelling) {
            (false, false) => ThreeValued::False,
            (false, true) => ThreeValued::Unknown,
            (true, true) => ThreeValued::True,
            (true, false) => {
                // do not panic here, there can be such a result for intermediate propositions
                ThreeValued::Unknown
            }
        }
    }
}
