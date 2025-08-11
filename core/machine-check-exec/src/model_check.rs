mod deduce;
mod property_checker;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use log::trace;
use machine_check_common::{
    check::Conclusion,
    property::{Property, Subproperty},
    ExecError, StateId, ThreeValued,
};
use mck::concr::FullMachine;

use crate::{model_check::property_checker::PropertyChecker, space::StateSpace};

use self::deduce::deduce_culprit;

use std::fmt::Debug;

#[derive(Debug)]
/// Three-valued model checker.
pub struct ThreeValuedChecker {
    property_checkers: HashMap<Property, PropertyChecker>,
}

impl ThreeValuedChecker {
    pub fn new() -> Self {
        Self {
            property_checkers: HashMap::new(),
        }
    }

    pub fn check_subproperty_with_labelling<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        subproperty: &Subproperty,
    ) -> Result<(Conclusion, BTreeMap<StateId, ThreeValued>), ExecError> {
        let property = subproperty.property();
        let conclusion = self.check_property(space, property)?;

        let property_checker = self
            .property_checkers
            .get_mut(property)
            .expect("Property checker should be inserted after the property was checked");

        // get the labelling as well
        let subproperty_index = subproperty.index();
        let mut labelling = BTreeMap::new();
        let getter = property_checker.last_getter(space);
        for state_id in space.states() {
            getter.cache_labelling(subproperty_index, state_id)?;
            let timed = getter.cache_and_get(subproperty_index, state_id)?;
            labelling.insert(state_id, timed.value.valuation);
        }
        Ok((conclusion, labelling))
    }

    /// Model-checks a mu-calculus proposition.
    pub fn check_property<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        property: &Property,
    ) -> Result<Conclusion, ExecError> {
        trace!("Checking property {:#?}", property);

        if !self.property_checkers.contains_key(property) {
            self.property_checkers
                .insert(property.clone(), PropertyChecker::new(property.clone()));
        }

        let property_checker = self
            .property_checkers
            .get_mut(property)
            .expect("Property checker should be just inserted");

        let result = property_checker.compute_interpretation(space)?;

        if !space.is_valid() {
            return Ok(Conclusion::NotCheckable);
        }

        // compute optimistic and pessimistic interpretation and get the conclusion from that
        match result {
            ThreeValued::False => Ok(Conclusion::Known(false)),
            ThreeValued::True => Ok(Conclusion::Known(true)),
            ThreeValued::Unknown => Ok(Conclusion::Unknown(deduce_culprit(
                property_checker,
                space,
                property,
            )?)),
        }
    }

    pub fn declare_regeneration<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        new_states: &BTreeSet<StateId>,
        changed_successors: &BTreeSet<StateId>,
    ) {
        let mut open_states = new_states.clone();
        open_states.extend(changed_successors.iter().cloned());

        let mut purge_states = BTreeSet::new();

        while let Some(state_id) = open_states.pop_last() {
            for predecessor_id in space.direct_predecessor_iter(state_id.into()) {
                let Ok(predecessor_id) = StateId::try_from(predecessor_id) else {
                    continue;
                };
                if !purge_states.contains(&predecessor_id) {
                    open_states.insert(predecessor_id);
                }
            }
            purge_states.insert(state_id);
        }

        trace!(
            "Declaring regeneration, new states: {:?}, changed successors: {:?}, purging states: {:?}",
            new_states,
            changed_successors,
            purge_states
        );

        // TODO: rework so that full recomputation is not necessary each time
        //self.property_checkers.clear();

        for property_checker in self.property_checkers.values_mut() {
            property_checker.purge_states(space, &purge_states);
        }
    }

    pub fn double_check<M: FullMachine>(&self, space: &StateSpace<M>) -> Result<(), ExecError> {
        for property_checker in self.property_checkers.values() {
            property_checker.double_check(space)?;
        }
        Ok(())
    }
}
