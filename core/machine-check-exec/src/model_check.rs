mod classic;
mod deduce;

use std::collections::BTreeMap;

use machine_check_common::{
    check::{Conclusion, PreparedProperty},
    property::Property,
    ExecError, StateId, ThreeValued,
};
use mck::concr::FullMachine;

use crate::space::StateSpace;

use self::{classic::ClassicChecker, deduce::deduce_culprit};

/// Perform three-valued model checking.
///
/// The proposition must be prepared beforehand.
pub(super) fn check_property<M: FullMachine>(
    space: &StateSpace<M>,
    prop: &PreparedProperty,
) -> Result<Conclusion, ExecError> {
    let mut checker = ThreeValuedChecker::new(space);
    checker.check_property(prop)
}

pub(super) fn check_property_with_labelling<M: FullMachine>(
    space: &StateSpace<M>,
    property: &PreparedProperty,
) -> Result<(Conclusion, BTreeMap<StateId, ThreeValued>), ExecError> {
    let mut checker = ThreeValuedChecker::new(space);
    let conclusion = checker.check_property(property)?;
    let labelling = checker.compute_property_labelling(property)?;
    Ok((conclusion, labelling))
}

/// Three-valued model checker.
struct ThreeValuedChecker<'a, M: FullMachine> {
    space: &'a StateSpace<M>,
    pessimistic: ClassicChecker<'a, M>,
    optimistic: ClassicChecker<'a, M>,
}

impl<'a, M: FullMachine> ThreeValuedChecker<'a, M> {
    fn new(space: &'a StateSpace<M>) -> Self {
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
        if !self.space.is_valid() {
            return Ok(Conclusion::NotCheckable);
        }

        let prop = &property.prepared();
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
    ) -> Result<BTreeMap<StateId, ThreeValued>, ExecError> {
        let prop = &property.prepared();
        // compute the optimistic and pessimistic interpretation labellings
        let pessimistic_labelling = self.pessimistic.compute_and_get_labelling(prop)?;
        let optimistic_labelling = self.optimistic.compute_and_get_labelling(prop)?;

        let mut result = BTreeMap::new();

        for node_id in self.space.nodes() {
            let Ok(state_id) = StateId::try_from(node_id) else {
                continue;
            };

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
