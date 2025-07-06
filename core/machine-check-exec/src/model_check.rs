mod deduce;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use log::{log_enabled, trace};
use machine_check_common::{
    check::Conclusion,
    property::{BiLogicOperator, NextOperator, Property, PropertyType, Subproperty},
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

    println!(
        "Computing the labelling for {:?}, check map: {:?}",
        subproperty, checker.check_map
    );

    // get the labelling as well
    let subproperty_index = subproperty.index();
    let _updated = checker.compute_labelling(subproperty_index)?;
    println!("Getting the labelling, check map: {:?}", checker.check_map);
    let labelling = checker.get_labelling(subproperty_index).clone();
    println!("Got the labelling");
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
        if let std::collections::hash_map::Entry::Vacant(e) =
            self.check_map.entry(subproperty_index)
        {
            // make all states dirty by default
            e.insert(CheckInfo {
                labelling: BTreeMap::new(),
                reasons: BTreeMap::new(),
                dirty: BTreeSet::from_iter(self.space.states()),
            });
        }

        //println!("Property: {:?}", self.property);
        //println!("Computing labelling for index {}", subproperty_index);

        let subproperty_entry = self.property.subproperty_entry(subproperty_index);

        match &subproperty_entry.ty {
            PropertyType::Const(c) => {
                let check_info = self.get_check_info_mut(subproperty_index);

                let constant = ThreeValued::from_bool(*c);
                // make dirty constant labelling
                for state_id in &check_info.dirty {
                    check_info.labelling.insert(*state_id, constant);
                }
            }
            PropertyType::Atomic(atomic_property) => {
                let check_info = self
                    .check_map
                    .get_mut(&subproperty_index)
                    .expect("Check info for atomic property should be available");
                // get dirty from space
                for state_id in check_info.dirty.iter().copied() {
                    check_info.labelling.insert(
                        state_id,
                        self.space.atomic_label(atomic_property, state_id)?,
                    );
                }
            }
            PropertyType::Negation(inner) => {
                // negate everything dirty or updated
                let inner_updated = self.compute_labelling(*inner)?;
                let mut dirty = BTreeSet::new();
                dirty.append(&mut self.get_check_info_mut(subproperty_index).dirty);
                dirty.extend(inner_updated);

                let inner_labelling = self.get_labelling(*inner);

                let mut updated_labelling = BTreeMap::new();

                for state_id in dirty.iter().copied() {
                    let value = !*inner_labelling
                        .get(&state_id)
                        .expect("Negation should have inner state labelling");
                    updated_labelling.insert(state_id, value);
                }

                let check_info = self.get_check_info_mut(subproperty_index);
                check_info.dirty = dirty;
                check_info.labelling.extend(updated_labelling);
            }
            PropertyType::BiLogicOperator(op) => self.compute_binary_op(subproperty_index, op)?,
            PropertyType::NextOperator(op) => self.compute_next_labelling(subproperty_index, op)?,
            PropertyType::LeastFixedPoint(inner) => self.compute_fixed_point_op(
                subproperty_index,
                *inner,
                ThreeValued::from_bool(false),
            )?,
            PropertyType::GreatestFixedPoint(inner) => self.compute_fixed_point_op(
                subproperty_index,
                *inner,
                ThreeValued::from_bool(true),
            )?,
            PropertyType::FixedPointVariable(fixed_point) => {
                // update from the fixed point
                let mut dirty = BTreeSet::new();
                dirty.append(&mut self.get_check_info_mut(subproperty_index).dirty);
                dirty.extend(self.get_check_info_mut(*fixed_point).dirty.iter().copied());

                let fixed_point_labelling = self.get_labelling(*fixed_point);
                let mut updated_labelling = BTreeMap::new();

                //println!("Check map: {:?}", self.check_map);

                for state_id in dirty.iter().copied() {
                    let fixed_point_value = *fixed_point_labelling.get(&state_id).expect(
                        "Fixed-point variabvle computation should have state labelling available",
                    );
                    updated_labelling.insert(state_id, fixed_point_value);
                }

                let check_info = self.get_check_info_mut(subproperty_index);
                check_info.dirty = dirty;
                check_info.labelling.extend(updated_labelling);
            }
        };

        let check_info = self.get_check_info_mut(subproperty_index);
        let mut updated_states = BTreeSet::new();
        std::mem::swap(&mut check_info.dirty, &mut updated_states);

        if log_enabled!(log::Level::Trace) {
            trace!(
                "Computed subproperty {:?} labelling {:?}, updated {:?}",
                subproperty_entry,
                check_info.labelling,
                updated_states
            );
        }

        /*println!(
            "Computed subproperty {:?} labelling {:?}, updated {:?}",
            subproperty_entry, check_info.labelling, updated_states
        );*/

        Ok(updated_states)
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
        subproperty_index: usize,
        op: &BiLogicOperator,
    ) -> Result<(), ExecError> {
        let mut dirty = BTreeSet::new();
        dirty.append(&mut self.get_check_info_mut(subproperty_index).dirty);

        let a_updated = self.compute_labelling(op.a)?;
        let b_updated = self.compute_labelling(op.b)?;

        let a_labelling = self.get_labelling(op.a);
        let b_labelling = self.get_labelling(op.b);

        dirty.extend(a_updated);
        dirty.extend(b_updated);

        let mut updated_labelling = BTreeMap::new();

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

            updated_labelling.insert(state_id, result_value);
        }

        let check_info = self.get_check_info_mut(subproperty_index);
        check_info.dirty = dirty;
        check_info.labelling.extend(updated_labelling);

        Ok(())
    }

    fn compute_fixed_point_op(
        &mut self,
        fixed_point_index: usize,
        inner_index: usize,
        initial_value: ThreeValued,
    ) -> Result<(), ExecError> {
        let constant_labelling = self.constant_labelling(initial_value);

        // initialise fixed-point computation labelling

        //println!("Constant labelling: {:?}", constant_labelling);

        self.check_map
            .get_mut(&fixed_point_index)
            .expect("Fixed-point info should be in check map")
            .labelling = constant_labelling;

        //println!("Check map: {:?}", self.check_map);

        let mut all_updated = BTreeSet::new();

        //println!("Computing fixed point");

        // compute inner property labelling and update variable labelling until they match
        loop {
            let updated = self.compute_labelling(inner_index)?;

            //println!("Updated in this iteration: {:?}", updated);

            all_updated.extend(updated.iter().cloned());

            if updated.is_empty() {
                // fixed-point reached
                let inner_labelling = self
                    .check_map
                    .get(&inner_index)
                    .expect("Check map should contain inner property")
                    .labelling
                    .clone();
                let fixed_point_info = self
                    .check_map
                    .get_mut(&fixed_point_index)
                    .expect("Check map should contain fixed-point property");
                fixed_point_info.labelling = inner_labelling;
                fixed_point_info.dirty = all_updated;

                return Ok(());
            } else {
                let fixed_point_labelling = &self
                    .check_map
                    .get(&fixed_point_index)
                    .expect("Check map should contain inner property")
                    .labelling;

                //println!("Variable labelling: {:?}", variable_labelling);

                let mut updated_labels = BTreeMap::new();
                for state_id in updated.iter().cloned() {
                    let previous = fixed_point_labelling
                        .get(&state_id)
                        .expect("Variable labelling should contain updated state");
                    let current_labelling = &self
                        .check_map
                        .get(&inner_index)
                        .expect("Check map should contain inner property")
                        .labelling;
                    let current = current_labelling
                        .get(&state_id)
                        .expect("Inner labelling should contain updated state");
                    if current != previous {
                        updated_labels.insert(state_id, *current);
                    }
                }

                // update the labelling and make updated dirty in the variable
                let fixed_point_info = self
                    .check_map
                    .get_mut(&fixed_point_index)
                    .expect("Check map should contain variable property");

                //println!("Really changed: {:?}", updated_labels);

                fixed_point_info.dirty = updated;
                fixed_point_info.labelling.extend(updated_labels);
            }
        }
    }

    fn compute_next_labelling(
        &mut self,
        subproperty_index: usize,
        op: &NextOperator,
    ) -> Result<(), ExecError> {
        let ground_value = ThreeValued::from_bool(op.is_universal);

        let check_info = &mut self.get_check_info_mut(subproperty_index);
        let mut current_reasons = BTreeMap::new();
        current_reasons.append(&mut check_info.reasons);

        let mut dirty = BTreeSet::new();
        dirty.append(&mut check_info.dirty);

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

        let check_info = &mut self.get_check_info_mut(subproperty_index);
        let mut previous_dirty_labels = BTreeMap::new();
        for state_id in dirty.iter().copied() {
            previous_dirty_labels.insert(state_id, check_info.labelling.get(&state_id).copied());
        }

        let inner_labelling = self.get_labelling(op.inner);

        //println!("Next dirty states: {:?}", dirty);

        //println!("Previous reasons: {:?}", reasons);

        let mut current_labelling = BTreeMap::new();

        // For each state in dirty states, compute the new value from the successors.
        let mut updated = BTreeSet::new();
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
                current_reasons.entry(dirty_id).or_insert(reason);
            }

            let update = if let Some(previous_label) = previous_dirty_labels
                .get(&dirty_id)
                .expect("Dirty label should be in previous dirty labels")
            {
                *previous_label != label
            } else {
                true
            };

            if update {
                current_labelling.insert(dirty_id, label);
                updated.insert(dirty_id);
            }
        }

        let check_info = self.get_check_info_mut(subproperty_index);
        check_info.labelling.extend(current_labelling);
        check_info.reasons = current_reasons;
        check_info.dirty = updated;

        //println!("Next valuations: {:?}", labelling);
        //println!("Next reasons: {:?}", reasons);

        Ok(())
    }

    fn constant_labelling(&self, value: ThreeValued) -> BTreeMap<StateId, ThreeValued> {
        BTreeMap::from_iter(self.space.nodes().filter_map(|node_id| {
            let Ok(state_id) = StateId::try_from(node_id) else {
                return None;
            };
            Some((state_id, value))
        }))
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
