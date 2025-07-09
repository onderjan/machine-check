mod deduce;

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap},
};

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

/// Three-valued model checker.
pub struct ThreeValuedChecker {
    property_checkers: HashMap<Property, PropertyChecker>,
}

#[derive(Debug)]
pub struct PropertyChecker {
    check_map: HashMap<usize, CheckInfo>,
}

#[derive(Debug)]
struct CheckInfo {
    labelling: BTreeMap<StateId, Label>,
    dirty: BTreeSet<StateId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Label {
    wave: u64,
    value: ThreeValued,
    next_states: Vec<StateId>,
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
        let _updated = property_checker.compute_labelling(property, space, subproperty_index)?;
        //println!("Getting the labelling, check map: {:?}", checker.check_map);
        let labelling = property_checker
            .get_labelling(subproperty_index)
            .iter()
            .map(|(state_id, label)| (*state_id, label.value))
            .collect();
        //println!("Got the labelling");
        Ok((conclusion, labelling))
    }

    /// Model-checks a mu-calculus proposition.
    pub fn check_property<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        property: &Property,
    ) -> Result<Conclusion, ExecError> {
        if !self.property_checkers.contains_key(property) {
            self.property_checkers.insert(
                property.clone(),
                PropertyChecker {
                    check_map: HashMap::new(),
                },
            );
        }

        let property_checker = self
            .property_checkers
            .get_mut(property)
            .expect("Property checker should be just inserted");

        let result = property_checker.compute_interpretation(space, property)?;

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

    pub fn declare_regeneration(
        &mut self,
        _new_states: &BTreeSet<StateId>,
        _changed_successors: &BTreeSet<StateId>,
    ) {
        // TODO: rework so that full recomputation is not necessary each time
        self.property_checkers.clear();
        /*println!(
            "Declared regeneration, new states: {:?}, changed successors: {:?}",
            new_states, changed_successors
        );*/
        /*for (property, property_checker) in &mut self.property_checkers {
            for (subproperty_id, check_info) in &mut property_checker.check_map {
                check_info.dirty.extend(new_states);
                let subproperty = property.subproperty_entry(*subproperty_id);
                //if matches!(subproperty.ty, PropertyType::Next(_)) {
                /*check_info.dirty.extend(changed_successors);
                for state_id in changed_successors {
                    check_info.reasons.remove(state_id);
                }*/
                check_info.dirty.clear();
                check_info.reasons.clear();
                check_info.labelling.clear();
                //}
            }
        }*/
    }
}

impl PropertyChecker {
    fn compute_interpretation<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        property: &Property,
    ) -> Result<ThreeValued, ExecError> {
        let _updated = self.compute_labelling(property, space, 0)?;
        let labelling = self.get_labelling(0);
        // conventionally, the property must hold in all initial states
        let mut result = ThreeValued::True;
        for initial_state_id in space.initial_iter() {
            let state_labelling = labelling
                .get(&initial_state_id)
                .expect("Labelling should contain initial state");
            result = result & state_labelling.value;
        }
        Ok(result)
    }

    fn compute_labelling<M: FullMachine>(
        &mut self,
        property: &Property,
        space: &StateSpace<M>,
        subproperty_index: usize,
    ) -> Result<BTreeMap<StateId, Label>, ExecError> {
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
                    dirty: BTreeSet::new(),
                },
            );
            // make all states dirty by default
            BTreeSet::from_iter(space.states())
        };

        //println!("Property: {:?}", self.property);
        //println!("Computing labelling for index {}", subproperty_index);

        let subproperty_entry = property.subproperty_entry(subproperty_index);

        let mut update = BTreeMap::new();

        match &subproperty_entry.ty {
            PropertyType::Const(c) => {
                let constant = ThreeValued::from_bool(*c);

                // make everything dirty have constant labelling
                for state_id in dirty {
                    update.insert(
                        state_id,
                        Label {
                            wave: 0,
                            value: constant,
                            next_states: Vec::new(),
                        },
                    );
                }
            }
            PropertyType::Atomic(atomic_property) => {
                for state_id in dirty {
                    let value = space.atomic_label(atomic_property, state_id)?;
                    update.insert(
                        state_id,
                        Label {
                            wave: 0,
                            value,
                            next_states: Vec::new(),
                        },
                    );
                }
            }
            PropertyType::Negation(inner) => {
                // if negation is dirty, inner must be dirty as well
                // it suffices to negate everything updated
                update = self.compute_labelling(property, space, *inner)?;
                for (_state_id, label_update) in update.iter_mut() {
                    label_update.value = !label_update.value;
                }
            }
            PropertyType::BiLogic(op) => {
                // if binary operator is dirty, inner must be dirty as well
                // it suffices to negate everything updated
                self.compute_binary_op(space, property, &mut update, op)?;
            }
            PropertyType::Next(op) => {
                self.compute_next_labelling(space, property, dirty, &mut update, op)?;
            }
            PropertyType::FixedPoint(op) => {
                return self.compute_fixed_point_op(space, property, subproperty_index, dirty, op);
            }
            PropertyType::FixedVariable(fixed_point) => {
                // update from the fixed point
                dirty.extend(
                    self.get_check_info_mut(property, *fixed_point)
                        .dirty
                        .iter()
                        .copied(),
                );

                let fixed_point_labelling = self.get_labelling(*fixed_point);

                for state_id in dirty {
                    let fixed_point_label = fixed_point_labelling.get(&state_id).expect(
                        "Fixed-point variable computation should have state labelling available",
                    ).clone();
                    let variable_label = Label {
                        wave: fixed_point_label.wave,
                        value: fixed_point_label.value,
                        next_states: Vec::new(),
                    };

                    update.insert(state_id, variable_label);
                }
            }
        };

        let check_info = self.get_check_info_mut(property, subproperty_index);

        let num_recomputed = update.len();

        let updated_states = Self::update_labelling(check_info, update);

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
        update: BTreeMap<StateId, Label>,
    ) -> BTreeMap<StateId, Label> {
        let mut updated_labels = BTreeMap::new();

        for (state_id, label) in update {
            if let Some(current_label) = check_info.labelling.get_mut(&state_id) {
                if label.value == current_label.value {
                    continue;
                }
                *current_label = label.clone();
            } else {
                check_info.labelling.insert(state_id, label.clone());
            }
            updated_labels.insert(state_id, label);
        }

        updated_labels
    }

    fn get_check_info_mut(
        &mut self,
        property: &Property,
        subproperty_index: usize,
    ) -> &mut CheckInfo {
        if let Some(info) = self.check_map.get_mut(&subproperty_index) {
            info
        } else {
            panic!(
                "Check info for the subproperty index {} of property {:?} should be available",
                subproperty_index, property
            )
        }
    }

    fn compute_binary_op<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        property: &Property,
        update: &mut BTreeMap<StateId, Label>,
        op: &BiLogicOperator,
    ) -> Result<(), ExecError> {
        let a_updated = self.compute_labelling(property, space, op.a)?;
        let b_updated = self.compute_labelling(property, space, op.b)?;

        let a_labelling = self.get_labelling(op.a);
        let b_labelling = self.get_labelling(op.b);

        let mut dirty = BTreeSet::from_iter(a_updated.keys());
        dirty.extend(b_updated.keys());

        for state_id in dirty {
            let a_update = a_updated.get(state_id).cloned().unwrap_or_else(|| {
                a_labelling
                    .get(state_id)
                    .cloned()
                    .expect("Binary operation should have left labelling available")
            });
            let b_update = b_updated.get(state_id).cloned().unwrap_or_else(|| {
                b_labelling
                    .get(state_id)
                    .cloned()
                    .expect("Binary operation should have right labelling available")
            });

            let a_value = a_update.value;
            let b_value = b_update.value;

            let result = if op.is_and {
                // we prefer the lesser value
                match a_value.cmp(&b_value) {
                    Ordering::Less => Some(a_update.clone()),
                    Ordering::Equal => None,
                    Ordering::Greater => Some(b_update.clone()),
                }
            } else {
                // we prefer the greater value
                match a_value.cmp(&b_value) {
                    Ordering::Less => Some(b_update.clone()),
                    Ordering::Equal => None,
                    Ordering::Greater => Some(a_update.clone()),
                }
            };

            let result = if let Some(result) = result {
                result
            } else if a_update.wave <= b_update.wave {
                a_update
            } else {
                b_update
            };

            update.insert(*state_id, result);
        }

        Ok(())
    }

    fn compute_next_labelling<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        property: &Property,
        mut dirty: BTreeSet<StateId>,
        update: &mut BTreeMap<StateId, Label>,
        op: &NextOperator,
    ) -> Result<(), ExecError> {
        let ground_value = ThreeValued::from_bool(op.is_universal);

        //let check_info = &mut self.get_check_info_mut(subproperty_index);
        //let mut current_reasons = BTreeMap::new();
        //current_reasons.append(&mut check_info.reasons);

        let inner_updated = self.compute_labelling(property, space, op.inner)?;

        // We need to compute states which are either dirty or the inner property was updated
        // for their direct successors.

        for state_id in inner_updated.keys() {
            //println!("Next updated state id: {}", state_id);
            for predecessor_id in space.direct_predecessor_iter((*state_id).into()) {
                if let Ok(predecessor_id) = StateId::try_from(predecessor_id) {
                    //println!("Considered state id: {}", predecessor_id);
                    dirty.insert(predecessor_id);
                }
            }
        }

        let inner_labelling = self.get_labelling(op.inner);

        //println!("Next dirty states: {:?}", dirty);

        //println!("Previous reasons: {:?}", reasons);

        //println!("Computing next for dirty {:?}", dirty);

        // For each state in dirty states, compute the new value from the successors.
        for dirty_id in dirty.iter().copied() {
            let mut dirty_label = inner_labelling
                .get(&dirty_id)
                .expect("Direct successor should labelled")
                .clone();
            let mut successors = Vec::from_iter(space.direct_successor_iter(dirty_id.into()).map(
                |successor_id| {
                    (
                        successor_id,
                        inner_labelling
                            .get(&successor_id)
                            .expect("Direct successor should labelled")
                            .clone(),
                    )
                },
            ));

            successors.sort_by(|a, b| a.1.wave.cmp(&b.1.wave));
            /*println!(
                "Computing next for {}, successors: {:?}",
                dirty_id, successors
            );*/

            for (successor_id, successor_label) in successors {
                let mut value = ground_value;
                if op.is_universal {
                    value = value & successor_label.value;
                } else {
                    value = value | successor_label.value;
                }

                if value != dirty_label.value {
                    let mut next_states = successor_label.next_states.clone();
                    next_states.push(successor_id);
                    dirty_label = Label {
                        wave: successor_label.wave,
                        value,
                        next_states,
                    };
                }
            }
            update.insert(dirty_id, dirty_label);
        }

        //println!("Next valuations: {:?}", update);
        //println!("Next reasons: {:?}", reasons);

        Ok(())
    }

    fn compute_fixed_point_op<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        property: &Property,
        fixed_point_index: usize,
        dirty: BTreeSet<StateId>,
        op: &FixedPointOperator,
    ) -> Result<BTreeMap<StateId, Label>, ExecError> {
        let ground_value = ThreeValued::from_bool(op.is_greatest);

        // initialise fixed-point computation labelling

        //println!("Constant labelling: {:?}", constant_labelling);

        // make sure all dirty states have some fixed-point labelling
        // and are shown as dirty when the variables look at them

        let check_info = self
            .check_map
            .get_mut(&fixed_point_index)
            .expect("Fixed-point info should be in check map");

        let mut all_updated = BTreeMap::new();

        // make sure that all dirty states are set to ground labelling at first so they can be completely recomputed

        let ground_labelling = Label {
            wave: 0,
            value: ground_value,
            next_states: Vec::new(),
        };

        for state_id in dirty.iter().copied() {
            check_info
                .labelling
                .insert(state_id, ground_labelling.clone());
            all_updated.insert(state_id, ground_labelling.clone());
        }

        //println!("Check map: {:?}", self.check_map);

        //println!("Computing fixed point");

        // compute inner property labelling and update variable labelling until they match
        loop {
            let mut current_update = self.compute_labelling(property, space, op.inner)?;

            //println!("Updated in this iteration: {:?}", current_update);

            let fixed_point_info = self
                .check_map
                .get_mut(&fixed_point_index)
                .expect("Check map should contain fixed-point property");

            if current_update.is_empty() {
                // fixed-point reached
                // the labelling now definitely corresponds to inner
                // just clear dirty as everything was computed

                fixed_point_info.dirty.clear();
                return Ok(all_updated);
            };

            for label in current_update.values_mut() {
                label.wave += 1;
            }

            // update the labelling and make updated dirty in the variable

            let updated = Self::update_labelling(fixed_point_info, current_update);
            fixed_point_info.dirty = BTreeSet::from_iter(updated.keys().copied());

            all_updated.extend(updated);

            //println!("Really changed: {:?}", updated);
        }
    }

    fn get_state_label(&self, subproperty_index: usize, state_index: StateId) -> &Label {
        // TODO: this is wasteful when looking at multiple states
        self.get_labelling(subproperty_index)
            .get(&state_index)
            .expect("Should contain state labelling")
    }

    fn get_state_root_label(&self, state_index: StateId) -> &Label {
        self.get_state_label(0, state_index)
    }

    fn get_labelling(&self, subproperty_index: usize) -> &BTreeMap<StateId, Label> {
        &self
            .check_map
            .get(&subproperty_index)
            .expect("Labelling should be present")
            .labelling
    }
}
