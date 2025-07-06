mod deduce;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use log::{log_enabled, trace};
use machine_check_common::{
    check::Conclusion,
    property::{
        BiLogicOperator, FixedPointOperator, NextOperator, Property, PropertyType, Subproperty,
    },
    ExecError, StateId, ThreeValued,
};
use mck::concr::FullMachine;

use crate::space::StateSpace;

use self::deduce::deduce_culprit;

/// Perform three-valued model checking.
///
/// The proposition must be prepared beforehand.
pub(super) fn check_property<M: FullMachine>(
    space: &StateSpace<M>,
    property: &Property,
) -> Result<Conclusion, ExecError> {
    let mut checker = ThreeValuedChecker::new(space, property);
    checker.check_property()
}

pub(super) fn check_subproperty_with_labelling<M: FullMachine>(
    space: &StateSpace<M>,
    subproperty: &Subproperty,
) -> Result<(Conclusion, BTreeMap<StateId, ThreeValued>), ExecError> {
    let mut checker = ThreeValuedChecker::new(space, subproperty.property());
    let conclusion = checker.check_property()?;

    /*println!(
        "Computing the labelling for {:?}, check map: {:?}",
        subproperty, checker.check_map
    );*/

    // get the labelling as well
    let subproperty_index = subproperty.index();
    let _updated = checker.compute_labelling(subproperty_index)?;
    //println!("Getting the labelling, check map: {:?}", checker.check_map);
    let labelling = checker.get_labelling(subproperty_index).clone();
    //println!("Got the labelling");
    Ok((conclusion, labelling))
}

/// Three-valued model checker.
struct ThreeValuedChecker<'a, M: FullMachine> {
    space: &'a StateSpace<M>,
    property: &'a Property,
    check_map: HashMap<usize, CheckInfo>,
}

#[derive(Debug)]
struct CheckInfo {
    labelling: BTreeMap<StateId, ThreeValued>,
    reasons: BTreeMap<StateId, StateId>,
    dirty: BTreeSet<StateId>,
}

impl<'a, M: FullMachine> ThreeValuedChecker<'a, M> {
    fn new(space: &'a StateSpace<M>, property: &'a Property) -> Self {
        Self {
            space,
            property,
            check_map: HashMap::new(),
        }
    }

    /// Model-checks a CTL proposition.
    ///
    /// The proposition must be prepared and made canonical beforehand.
    fn check_property(&mut self) -> Result<Conclusion, ExecError> {
        let result = self.compute_interpretation()?;

        if !self.space.is_valid() {
            return Ok(Conclusion::NotCheckable);
        }

        // compute optimistic and pessimistic interpretation and get the conclusion from that
        match result {
            ThreeValued::False => Ok(Conclusion::Known(false)),
            ThreeValued::True => Ok(Conclusion::Known(true)),
            ThreeValued::Unknown => Ok(Conclusion::Unknown(deduce_culprit(self)?)),
        }
    }

    fn compute_interpretation(&mut self) -> Result<ThreeValued, ExecError> {
        let _updated = self.compute_labelling(0)?;
        let labelling = self.get_labelling(0);
        // conventionally, the property must hold in all initial states
        let mut result = ThreeValued::True;
        for initial_state_id in self.space.initial_iter() {
            let state_labelling = labelling
                .get(&initial_state_id)
                .expect("Labelling should contain initial state");
            result = result & *state_labelling;
        }
        Ok(result)
    }

    fn compute_labelling(
        &mut self,
        subproperty_index: usize,
    ) -> Result<BTreeSet<StateId>, ExecError> {
        let mut dirty = if let Some(check_info) = self.check_map.get_mut(&subproperty_index) {
            // take all dirty states from info
            let mut dirty = BTreeSet::new();
            std::mem::swap(&mut dirty, &mut check_info.dirty);
            dirty
        } else {
            self.check_map.insert(
                subproperty_index,
                CheckInfo {
                    labelling: BTreeMap::new(),
                    reasons: BTreeMap::new(),
                    dirty: BTreeSet::new(),
                },
            );
            // make all states dirty by default
            BTreeSet::from_iter(self.space.states())
        };

        //println!("Property: {:?}", self.property);
        //println!("Computing labelling for index {}", subproperty_index);

        let subproperty_entry = self.property.subproperty_entry(subproperty_index);

        let mut update = BTreeMap::new();

        let reasons = match &subproperty_entry.ty {
            PropertyType::Const(c) => {
                let constant = ThreeValued::from_bool(*c);

                // make everything dirty have constant labelling
                for state_id in dirty {
                    update.insert(state_id, constant);
                }
                None
            }
            PropertyType::Atomic(atomic_property) => {
                for state_id in dirty {
                    update.insert(
                        state_id,
                        self.space.atomic_label(atomic_property, state_id)?,
                    );
                }
                None
            }
            PropertyType::Negation(inner) => {
                // negate everything dirty or updated
                let inner_updated = self.compute_labelling(*inner)?;
                dirty.extend(inner_updated);

                let inner_labelling = self.get_labelling(*inner);

                for state_id in dirty {
                    let value = !*inner_labelling
                        .get(&state_id)
                        .expect("Negation should have inner state labelling");
                    update.insert(state_id, value);
                }
                None
            }
            PropertyType::BiLogic(op) => {
                self.compute_binary_op(dirty, &mut update, op)?;
                None
            }
            PropertyType::Next(op) => Some(self.compute_next_labelling(dirty, &mut update, op)?),
            PropertyType::FixedPoint(op) => {
                return self.compute_fixed_point_op(subproperty_index, dirty, op);
            }
            PropertyType::FixedVariable(fixed_point) => {
                // update from the fixed point
                dirty.extend(self.get_check_info_mut(*fixed_point).dirty.iter().copied());

                let fixed_point_labelling = self.get_labelling(*fixed_point);

                for state_id in dirty {
                    let fixed_point_value = *fixed_point_labelling.get(&state_id).expect(
                        "Fixed-point variable computation should have state labelling available",
                    );
                    update.insert(state_id, fixed_point_value);
                }
                None
            }
        };

        let check_info = self.get_check_info_mut(subproperty_index);

        let num_recomputed = update.len();

        let updated_states = Self::update_labelling(check_info, update);

        if let Some(reasons) = reasons {
            // update reasons
            for updated_state in updated_states.iter().copied() {
                if let Some(reason) = reasons.get(&updated_state) {
                    check_info.reasons.insert(updated_state, *reason);
                }
            }
        }

        if log_enabled!(log::Level::Trace) {
            trace!(
                "Computed subproperty {:?} labelling {:?}, recomputed {}, updated {}",
                subproperty_entry,
                check_info.labelling,
                num_recomputed,
                updated_states.len()
            );
        }

        Ok(updated_states)
    }

    fn update_labelling(
        check_info: &mut CheckInfo,
        update: BTreeMap<StateId, ThreeValued>,
    ) -> BTreeSet<StateId> {
        let mut updated_states: BTreeSet<StateId> = BTreeSet::new();

        for (state_id, value) in update {
            if let Some(labelling_value) = check_info.labelling.get_mut(&state_id) {
                if value == *labelling_value {
                    continue;
                }
                *labelling_value = value;
            } else {
                check_info.labelling.insert(state_id, value);
            }
            updated_states.insert(state_id);
        }

        updated_states
    }

    fn get_check_info_mut(&mut self, subproperty_index: usize) -> &mut CheckInfo {
        if let Some(info) = self.check_map.get_mut(&subproperty_index) {
            info
        } else {
            panic!(
                "Check info for the subproperty index {} of property {:?} should be available",
                subproperty_index, self.property
            )
        }
    }

    fn compute_binary_op(
        &mut self,
        mut dirty: BTreeSet<StateId>,
        update: &mut BTreeMap<StateId, ThreeValued>,
        op: &BiLogicOperator,
    ) -> Result<(), ExecError> {
        let a_updated = self.compute_labelling(op.a)?;
        let b_updated = self.compute_labelling(op.b)?;

        let a_labelling = self.get_labelling(op.a);
        let b_labelling = self.get_labelling(op.b);

        dirty.extend(a_updated);
        dirty.extend(b_updated);

        for state_id in dirty.iter().copied() {
            let a_value = *a_labelling
                .get(&state_id)
                .expect("Binary operation should have left labelling available");
            let b_value = *b_labelling
                .get(&state_id)
                .expect("Binary operation should have right labelling available");

            let result_value = if op.is_and {
                a_value & b_value
            } else {
                a_value | b_value
            };

            update.insert(state_id, result_value);
        }

        Ok(())
    }

    fn compute_next_labelling(
        &mut self,
        mut dirty: BTreeSet<StateId>,
        update: &mut BTreeMap<StateId, ThreeValued>,
        op: &NextOperator,
    ) -> Result<BTreeMap<StateId, StateId>, ExecError> {
        let ground_value = ThreeValued::from_bool(op.is_universal);

        //let check_info = &mut self.get_check_info_mut(subproperty_index);
        //let mut current_reasons = BTreeMap::new();
        //current_reasons.append(&mut check_info.reasons);

        let inner_updated = self.compute_labelling(op.inner)?;

        // We need to compute states which are either dirty or the inner property was updated
        // for their direct successors.

        for state_id in &inner_updated {
            //println!("Next updated state id: {}", state_id);
            for predecessor_id in self.space.direct_predecessor_iter((*state_id).into()) {
                if let Ok(predecessor_id) = StateId::try_from(predecessor_id) {
                    //println!("Considered state id: {}", predecessor_id);
                    dirty.insert(predecessor_id);
                }
            }
        }

        let inner_labelling = self.get_labelling(op.inner);

        //println!("Next dirty states: {:?}", dirty);

        //println!("Previous reasons: {:?}", reasons);

        let mut reasons = BTreeMap::new();

        // For each state in dirty states, compute the new value from the successors.
        for dirty_id in dirty.iter().copied() {
            let mut label = ground_value;
            let mut reason = None;
            for successor_id in self.space.direct_successor_iter(dirty_id.into()) {
                let successor_value = inner_labelling
                    .get(&successor_id)
                    .expect("Direct successor should labelled");
                let old_label = label;
                if op.is_universal {
                    label = label & *successor_value;
                } else {
                    label = label | *successor_value;
                }

                if label != old_label && reason.is_none() {
                    reason = Some(successor_id);
                }
            }

            if let Some(reason) = reason {
                // insert reason if it does not exist already
                // TODO: this will not play well with updating dirty states from the outside
                reasons.entry(dirty_id).or_insert(reason);
            }

            update.insert(dirty_id, label);
        }

        //println!("Next valuations: {:?}", labelling);
        //println!("Next reasons: {:?}", reasons);

        Ok(reasons)
    }

    fn compute_fixed_point_op(
        &mut self,
        fixed_point_index: usize,
        dirty: BTreeSet<StateId>,
        op: &FixedPointOperator,
    ) -> Result<BTreeSet<StateId>, ExecError> {
        let ground_value = ThreeValued::from_bool(op.is_greatest);

        // initialise fixed-point computation labelling

        //println!("Constant labelling: {:?}", constant_labelling);

        // make sure all dirty states have some fixed-point labelling
        // and are shown as dirty when the variables look at them

        let check_info = self
            .check_map
            .get_mut(&fixed_point_index)
            .expect("Fixed-point info should be in check map");

        for state_id in dirty.iter().copied() {
            check_info.labelling.entry(state_id).or_insert(ground_value);
        }

        let mut all_updated = dirty.clone();
        check_info.dirty = dirty;

        //println!("Check map: {:?}", self.check_map);

        //println!("Computing fixed point");

        // compute inner property labelling and update variable labelling until they match
        loop {
            let inner_updated = self.compute_labelling(op.inner)?;

            //println!("Updated in this iteration: {:?}", updated);

            all_updated.extend(inner_updated.iter().cloned());

            if inner_updated.is_empty() {
                // fixed-point reached
                // the labelling now definitely corresponds to inner
                // just clear dirty as everything was computed
                let fixed_point_info = self
                    .check_map
                    .get_mut(&fixed_point_index)
                    .expect("Check map should contain fixed-point property");

                fixed_point_info.dirty.clear();
                return Ok(all_updated);
            };

            //println!("Variable labelling: {:?}", variable_labelling);

            let inner_labelling = &self
                .check_map
                .get(&op.inner)
                .expect("Check map should contain inner property")
                .labelling;

            let mut current_update = BTreeMap::new();
            for state_id in inner_updated.iter().cloned() {
                let current = inner_labelling
                    .get(&state_id)
                    .expect("Inner labelling should contain updated state");
                current_update.insert(state_id, *current);
            }

            // update the labelling and make updated dirty in the variable
            let fixed_point_info = self
                .check_map
                .get_mut(&fixed_point_index)
                .expect("Check map should contain variable property");

            let updated = Self::update_labelling(fixed_point_info, current_update);

            //println!("Really changed: {:?}", updated);

            fixed_point_info.dirty = updated;
        }
    }

    fn get_state_labelling(&self, subproperty_index: usize, state_index: StateId) -> ThreeValued {
        // TODO: this is wasteful when looking at multiple states
        *self
            .get_labelling(subproperty_index)
            .get(&state_index)
            .expect("Should contain state labelling")
    }

    fn get_state_root_labelling(&self, state_index: StateId) -> ThreeValued {
        self.get_state_labelling(0, state_index)
    }

    fn get_labelling(&self, subproperty_index: usize) -> &BTreeMap<StateId, ThreeValued> {
        &self
            .check_map
            .get(&subproperty_index)
            .expect("Labelling should be present")
            .labelling
    }

    fn get_reasons(&self, subproperty_index: usize) -> &BTreeMap<StateId, StateId> {
        &self
            .check_map
            .get(&subproperty_index)
            .expect("Reasons should be present")
            .reasons
    }
}
