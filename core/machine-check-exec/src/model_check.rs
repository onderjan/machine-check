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
    classic: ClassicChecker<'a, M>,
}

impl<'a, M: FullMachine> ThreeValuedChecker<'a, M> {
    fn new(space: &'a StateSpace<M>) -> Self {
        Self {
            space,
            classic: ClassicChecker::new(space),
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
        match self.classic.compute_interpretation(prop)? {
            ThreeValued::False => Ok(Conclusion::Known(false)),
            ThreeValued::True => Ok(Conclusion::Known(true)),
            ThreeValued::Unknown => Ok(Conclusion::Unknown(deduce_culprit(self, prop)?)),
        }
    }

    pub fn compute_property_labelling(
        &mut self,
        property: &PreparedProperty,
    ) -> Result<BTreeMap<StateId, ThreeValued>, ExecError> {
        let prop = &property.prepared();
        // compute the optimistic and pessimistic interpretation labellings
        Ok(self.classic.compute_and_get_labelling(prop)?.clone())
    }

    fn get_state_labelling(&self, prop: &Property, state_index: StateId) -> ThreeValued {
        let property_id = self
            .classic
            .get_property_id(prop)
            .expect("Should contain property when getting state labelling");
        *self
            .classic
            .get_labelling(property_id)
            .get(&state_index)
            .expect("Should contain state labelling")
    }

    fn get_state_labelling_reason(&self, prop: &Property, state_index: StateId) -> Option<StateId> {
        let property_id = self
            .classic
            .get_property_id(prop)
            .expect("Should contain property when getting state labelling reason");
        self.classic
            .get_reasons(property_id)
            .get(&state_index)
            .copied()
    }
}
