mod deduce;
mod nonincremental;
mod property_checker;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use log::trace;
use machine_check_common::{
    check::{Conclusion, KnownConclusion},
    iir::IProperty,
    ExecError, ParamValuation, StateId,
};
use mck::concr::FullMachine;

use crate::{model_check::property_checker::PropertyChecker, space::StateSpace};

use std::fmt::Debug;

#[derive(Debug)]
/// Three-valued model checker.
pub struct ThreeValuedChecker {
    property_checkers: HashMap<IProperty, PropertyChecker>,
}

impl ThreeValuedChecker {
    pub fn new() -> Self {
        Self {
            property_checkers: HashMap::new(),
        }
    }

    pub fn get_labellings<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        property: &IProperty,
    ) -> Result<BTreeMap<usize, BTreeMap<StateId, ParamValuation>>, ExecError> {
        let property_checker = self
            .property_checkers
            .get_mut(property)
            .expect("Property checker should be inserted after the property was checked");

        // get the labellings as well
        let mut labellings = BTreeMap::<usize, BTreeMap<StateId, ParamValuation>>::new();
        for subproperty_index in 0..property.num_subproperties() {
            let labelling = labellings.entry(subproperty_index).or_default();

            for state_id in space.states() {
                let timed = property_checker
                    .last_getter(space)
                    .compute_latest_timed(subproperty_index, state_id)?;
                labelling.insert(state_id, timed.value.valuation());
            }
        }

        Ok(labellings)
    }

    /// Model-checks a mu-calculus proposition.
    pub fn check_property<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        property: &IProperty,
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

        if result.is_known() {
            // double-check known result using the incremental algorithm non-incrementally
            property_checker.double_check(space)?;

            // triple-check known result using the non-incremental algorithm

            let basic_result = nonincremental::check_property(space, property)?;
            if result != basic_result {
                panic!(
                    "Result {:?} does not match the result of basic non-incremental algorithm {:?}",
                    result, basic_result
                );
            }
        }

        // compute optimistic and pessimistic interpretation and get the conclusion from that
        match result {
            ParamValuation::False => Ok(Conclusion::Known(KnownConclusion::False)),
            ParamValuation::True => Ok(Conclusion::Known(KnownConclusion::True)),
            ParamValuation::Dependent => Ok(Conclusion::Known(KnownConclusion::Dependent)),
            ParamValuation::Unknown => Ok(Conclusion::Unknown(deduce::deduce_culprit(
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

        let purge_states = open_states;

        trace!(
            "Declaring regeneration, new states: {:?}, changed successors: {:?}, purging states: {:?}",
            new_states,
            changed_successors,
            purge_states
        );

        for property_checker in self.property_checkers.values_mut() {
            property_checker.purge_states(space, &purge_states);
        }
    }

    pub fn remove_states(&mut self, removed_states: &BTreeSet<StateId>) {
        for property_checker in self.property_checkers.values_mut() {
            property_checker.remove_states(removed_states);
        }
    }
}
