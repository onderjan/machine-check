use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use log::{log_enabled, trace};
use machine_check_common::{
    check::Property,
    property::{BiLogicOperator, FixedPointOperator, NextOperator, PropertyType},
    ExecError, StateId, ThreeValued,
};
use mck::concr::FullMachine;

use crate::{
    model_check::history::{HistoryIndex, HistoryPoint, Label},
    space::StateSpace,
};

#[derive(Debug)]
pub struct PropertyChecker {
    check_map: BTreeMap<usize, CheckInfo>,
    very_dirty: BTreeSet<StateId>,
}

#[derive(Debug)]
struct CheckInfo {
    pub labelling: BTreeMap<StateId, Label>,
    pub dirty: BTreeSet<StateId>,
    pub fixed_reaches: BTreeSet<HistoryIndex>,
}

impl PropertyChecker {
    pub fn new() -> Self {
        Self {
            check_map: BTreeMap::new(),
            very_dirty: BTreeSet::new(),
        }
    }

    pub fn purge_states(&mut self, purge_states: &BTreeSet<StateId>) {
        self.very_dirty.extend(purge_states.iter());

        for check_info in self.check_map.values_mut() {
            check_info.dirty.extend(purge_states.iter());

            for state_id in purge_states {
                check_info.labelling.remove(state_id);
            }
        }
    }

    pub fn compute_interpretation<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        property: &Property,
    ) -> Result<ThreeValued, ExecError> {
        let _updated = self.compute_labelling(property, space, 0, &mut HistoryIndex(0))?;
        self.very_dirty.clear();
        let labelling = self.get_labelling(0);
        // conventionally, the property must hold in all initial states
        let mut result = ThreeValued::True;
        for initial_state_id in space.initial_iter() {
            let label = labelling
                .get(&initial_state_id)
                .expect("Labelling should contain initial state");
            let state_value = label.last_point().value;

            result = result & state_value;
        }

        if log_enabled!(log::Level::Trace) {
            trace!("Computed interpretation of {:?}", property);

            for (subproperty_index, check_info) in &self.check_map {
                let subproperty = property.subproperty_entry(*subproperty_index);

                let mut display = format!(
                    "Subproperty {} ({:?}): resets {:?}, labelling [\n",
                    subproperty_index, subproperty, check_info.fixed_reaches
                );
                for (state_id, label) in &check_info.labelling {
                    display.push_str(&format!("\t{}: {:?}\n", state_id, label));
                }
                display.push_str("]\n");

                trace!("{}", display);
            }
        }

        Ok(result)
    }

    pub fn compute_labelling<M: FullMachine>(
        &mut self,
        property: &Property,
        space: &StateSpace<M>,
        subproperty_index: usize,
        history_index: &mut HistoryIndex,
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
                    fixed_reaches: BTreeSet::new(),
                },
            );
            // make all states dirty by default
            BTreeSet::from_iter(space.states())
        };

        dirty.extend(self.very_dirty.iter());

        let dirty_clone = if log_enabled!(log::Level::Trace) {
            Some(dirty.clone())
        } else {
            None
        };

        //println!("Property: {:?}", self.property);
        //println!("Computing labelling for index {}", subproperty_index);

        let subproperty_entry = property.subproperty_entry(subproperty_index);

        //println!("Computing labelling for {:?}", subproperty_entry);

        let mut update = BTreeMap::new();

        match &subproperty_entry.ty {
            PropertyType::Const(c) => {
                let constant = ThreeValued::from_bool(*c);

                // make everything dirty have constant labelling
                for state_id in dirty {
                    update.insert(state_id, Label::constant(constant));
                }
            }
            PropertyType::Atomic(atomic_property) => {
                for state_id in dirty {
                    let value = space.atomic_label(atomic_property, state_id)?;
                    update.insert(state_id, Label::constant(value));
                }
            }
            PropertyType::Negation(inner) => {
                // if negation is dirty, inner must be dirty as well
                // it suffices to negate everything updated
                update = self.compute_labelling(property, space, *inner, history_index)?;
                for (_state_id, label_update) in update.iter_mut() {
                    for history_point in label_update.history.values_mut() {
                        history_point.value = !history_point.value;
                    }
                }
            }
            PropertyType::BiLogic(op) => {
                // if binary operator is dirty, inner must be dirty as well
                // it suffices to negate everything updated
                self.compute_binary_op(space, property, history_index, &mut update, op)?;
            }
            PropertyType::Next(op) => {
                self.compute_next_labelling(
                    space,
                    property,
                    history_index,
                    dirty,
                    &mut update,
                    op,
                )?;
            }
            PropertyType::FixedPoint(op) => {
                return self.compute_fixed_point_op(
                    space,
                    property,
                    history_index,
                    subproperty_index,
                    dirty,
                    op,
                );
            }
            PropertyType::FixedVariable(fixed_point) => {
                // update from the fixed point
                dirty.extend(
                    self.check_map
                        .get_mut(fixed_point)
                        .expect("Check info should be available")
                        .dirty
                        .iter()
                        .copied(),
                );

                trace!("Dirty states for fixed variable: {:?}", dirty);

                let fixed_point_labelling = self.get_labelling(*fixed_point);

                for state_id in dirty {
                    let mut variable_label = fixed_point_labelling.get(&state_id).expect(
                        "Fixed-point variable computation should have state labelling available",
                    ).clone();

                    for history_point in variable_label.history.values_mut() {
                        history_point.next_states = Vec::new();
                    }

                    update.insert(state_id, variable_label);
                }

                /*
                for state_id in dirty {
                    let mut variable_label = fixed_point_labelling.get(&state_id).expect(
                        "Fixed-point variable computation should have state labelling available",
                    ).clone();

                    for history_point in variable_label.history.values_mut() {
                        history_point.next_states = Vec::new();
                    }

                    update.insert(state_id, variable_label);
                }*/
            }
        };

        let check_info = self
            .check_map
            .get_mut(&subproperty_index)
            .expect("Check info should be available");

        let updated_states = Self::update_labelling(check_info, update, &self.very_dirty);

        if log_enabled!(log::Level::Trace) {
            trace!(
                "{:?}: Computed subproperty {} ({:?}) dirty {:?}, update {:#?}",
                history_index,
                subproperty_index,
                subproperty_entry,
                dirty_clone.unwrap(),
                updated_states
            );
        }

        /*println!(
            "{:?}: computed subproperty {:?} updated {:#?}",
            history_index, subproperty_entry, updated_states
        );*/

        Ok(updated_states)
    }

    fn update_labelling(
        check_info: &mut CheckInfo,
        update: BTreeMap<StateId, Label>,
        very_dirty: &BTreeSet<StateId>,
    ) -> BTreeMap<StateId, Label> {
        let mut updated_labels = BTreeMap::new();

        for (state_id, new_label) in update {
            let updated_label = if let Some(current_label) = check_info.labelling.get_mut(&state_id)
            {
                if new_label == *current_label {
                    if very_dirty.contains(&state_id) {
                        updated_labels.insert(state_id, new_label);
                    }
                    continue;
                }

                let mut updated_label = false;

                for (history_index, history_point) in new_label.history {
                    if let Some(prev_history_point) = current_label.history.get(&history_index) {
                        assert!(*prev_history_point == history_point);
                        continue;
                    };

                    current_label.history.insert(history_index, history_point);
                    updated_label = true;
                }

                if !updated_label {
                    continue;
                }

                current_label.clone()
            } else {
                check_info.labelling.insert(state_id, new_label.clone());
                new_label
            };
            updated_labels.insert(state_id, updated_label);
        }

        updated_labels
    }

    fn compute_binary_op<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        property: &Property,
        history_index: &mut HistoryIndex,
        update: &mut BTreeMap<StateId, Label>,
        op: &BiLogicOperator,
    ) -> Result<(), ExecError> {
        let a_updated = self.compute_labelling(property, space, op.a, history_index)?;
        let b_updated = self.compute_labelling(property, space, op.b, history_index)?;

        let a_labelling = self.get_labelling(op.a);
        let b_labelling = self.get_labelling(op.b);

        let mut dirty = BTreeSet::from_iter(a_updated.keys());
        dirty.extend(b_updated.keys());

        for state_id in dirty {
            let a_label = a_updated.get(state_id).cloned().unwrap_or_else(|| {
                a_labelling
                    .get(state_id)
                    .cloned()
                    .expect("Binary operation should have left labelling available")
            });
            let b_label = b_updated.get(state_id).cloned().unwrap_or_else(|| {
                b_labelling
                    .get(state_id)
                    .cloned()
                    .expect("Binary operation should have right labelling available")
            });

            let mut history_indices = BTreeSet::new();
            for key in a_label.history.keys().chain(b_label.history.keys()) {
                if !history_indices.contains(key) {
                    history_indices.insert(*key);
                }
            }

            let mut result_history = BTreeMap::new();

            let a_point = a_label.at_history_index(history_index);
            let b_point = b_label.at_history_index(history_index);

            let a_value = a_point.value;
            let b_value = b_point.value;

            let result_point = if op.is_and {
                // we prefer the lesser value
                match a_value.cmp(&b_value) {
                    Ordering::Less => a_point,
                    Ordering::Equal => a_point,
                    Ordering::Greater => b_point,
                }
            } else {
                // we prefer the greater value
                match a_value.cmp(&b_value) {
                    Ordering::Less => b_point,
                    Ordering::Equal => a_point,
                    Ordering::Greater => a_point,
                }
            };

            result_history.insert(*history_index, result_point.clone());

            update.insert(
                *state_id,
                Label {
                    history: result_history,
                },
            );
        }

        Ok(())
    }

    fn compute_next_labelling<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        property: &Property,
        history_index: &mut HistoryIndex,
        mut dirty: BTreeSet<StateId>,
        update: &mut BTreeMap<StateId, Label>,
        op: &NextOperator,
    ) -> Result<(), ExecError> {
        let ground_value = ThreeValued::from_bool(op.is_universal);

        let inner_updated = self.compute_labelling(property, space, op.inner, history_index)?;

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
            //let mut history_indices = BTreeSet::new();

            let direct_successors = BTreeMap::from_iter(
                space
                    .direct_successor_iter(dirty_id.into())
                    .map(|state_id| {
                        (
                            state_id,
                            inner_labelling
                                .get(&state_id)
                                .expect("Direct successor should labelled")
                                .clone(),
                        )
                    }),
            );

            /*for successor_label in direct_successors.values() {
                for history_index in successor_label.history.keys() {
                    if !history_indices.contains(history_index) {
                        history_indices.insert(*history_index);
                    }
                }
            }*/

            let mut result_history = BTreeMap::new();

            //println!("Computing next for history indices {:?}", history_indices);

            //for history_index in history_indices {
            let mut history_point = HistoryPoint {
                value: ground_value,
                next_states: Vec::new(),
            };

            let mut history_sequence = Vec::new();

            for (successor_id, successor_label) in &direct_successors {
                /*println!(
                    "Getting history point {:?} for {:?}",
                    history_point, successor_id
                );*/
                let (history_index, history_point) =
                    successor_label.at_history_index_key_value(history_index);
                history_sequence.push((*successor_id, *history_index, history_point.clone()));
            }
            //println!("Got history points");

            history_sequence.sort_by(|(a_id, a_index, _a_point), (b_id, b_index, _b_point)| {
                (a_index, a_id).cmp(&(b_index, b_id))
            });

            for (successor_id, _successor_index, successor_point) in &history_sequence {
                let new_value = if op.is_universal {
                    history_point.value & successor_point.value
                } else {
                    history_point.value | successor_point.value
                };

                if history_point.value != new_value {
                    history_point.value = new_value;
                    history_point.next_states = successor_point.next_states.clone();
                    history_point.next_states.push(*successor_id);
                }
            }
            result_history.insert(*history_index, history_point);
            //}

            update.insert(
                dirty_id,
                Label {
                    history: result_history,
                },
            );
        }

        //println!("Next valuations: {:?}", update);
        //println!("Next reasons: {:?}", reasons);

        Ok(())
    }

    fn compute_fixed_point_op<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        property: &Property,
        history_index: &mut HistoryIndex,
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

        let next_fixed_reach = check_info
            .fixed_reaches
            .range((
                std::ops::Bound::Excluded(*history_index),
                std::ops::Bound::Unbounded,
            ))
            .next()
            .cloned();

        trace!(
            "Fixed reaches: {:?}, current history index: {}, next fixed reach: {:?}",
            check_info.fixed_reaches,
            history_index.0,
            next_fixed_reach
        );

        let mut all_updated = BTreeMap::new();

        // make sure that all dirty states are set to ground labelling at first within our history index so they can be completely recomputed

        let ground_history_point = HistoryPoint {
            next_states: Vec::new(),
            value: ground_value,
        };

        let ground_update = BTreeMap::new();

        for state_id in dirty.iter().cloned() {
            let label = check_info
                .labelling
                .entry(state_id)
                .or_insert_with(|| Label {
                    history: BTreeMap::new(),
                });
            label
                .history
                .insert(*history_index, ground_history_point.clone());
        }

        Self::update_labelling(check_info, ground_update, &self.very_dirty);

        //println!("Check map: {:?}", self.check_map);

        //println!("Computing fixed point");

        // compute inner property labelling and update variable labelling until they match
        loop {
            let inner_update = self.compute_labelling(property, space, op.inner, history_index)?;

            let prev_history_index = *history_index;

            history_index.0 += 1;

            //println!("Updated in this iteration: {:?}", current_update);

            let fixed_point_info = self
                .check_map
                .get_mut(&fixed_point_index)
                .expect("Check map should contain fixed-point property");

            let mut current_update = BTreeMap::new();

            for (state_id, label) in inner_update {
                /*
                let mut history = BTreeMap::new();
                history.append(&mut label.history);
                *label = fixed_point_info
                    .labelling
                    .get(state_id)
                    .expect("Updated fixed-point state should have a labelling")
                    .clone();

                for (mut history_index, history_point) in history {
                    let change_occured = if let Some(prev_point) = label.history.get(&history_index)
                    {
                        *prev_point != history_point
                    } else {
                        true
                    };
                    if change_occured {
                        history_index.0 += 1;
                        label.history.insert(history_index, history_point);
                    }
                }*/

                let fixed_point_label = fixed_point_info
                    .labelling
                    .get(&state_id)
                    .expect("Fixed point labelling should have updated state");

                let prev_fixed_point = fixed_point_label.at_history_index(&prev_history_index);
                let current_fixed_point = label.at_history_index(&*history_index);

                if prev_fixed_point != current_fixed_point {
                    /*println!(
                        "Prev fixed point: {:?}, current fixed point: {:?}",
                        prev_fixed_point, current_fixed_point
                    );*/

                    let current_fixed_point = current_fixed_point.clone();
                    let mut label = fixed_point_label.clone();
                    label.history.insert(*history_index, current_fixed_point);
                    current_update.insert(state_id, label);
                }
            }

            // update the labelling and make updated dirty in the variable

            let updated =
                Self::update_labelling(fixed_point_info, current_update, &self.very_dirty);

            fixed_point_info.dirty = BTreeSet::from_iter(updated.keys().copied());

            all_updated.extend(updated);

            if let Some(next_fixed_reach) = next_fixed_reach {
                if next_fixed_reach.0 > history_index.0 {
                    trace!(
                        "Extending fixed-point dirty {:?} with initial dirty {:?} as {} > {}",
                        fixed_point_info.dirty,
                        dirty,
                        next_fixed_reach.0,
                        history_index.0
                    );
                    fixed_point_info.dirty.extend(dirty.iter());
                }
            }

            /*println!(
                "{:?}: computed fixed point, lowest updated index: {:?}, updated {:#?}",
                history_index, lowest_updated_index, updated
            );*/

            if fixed_point_info.dirty.is_empty() {
                // fixed-point reached
                // the labelling now definitely corresponds to inner
                // just clear dirty as everything was computed

                //println!("Reached fixed point at {:?}", history_index);

                fixed_point_info.fixed_reaches.insert(*history_index);
                return Ok(all_updated);
            };

            //println!("Really changed: {:?}", updated);
        }
    }

    pub fn get_state_label(&self, subproperty_index: usize, state_index: StateId) -> &Label {
        // TODO: this is wasteful when looking at multiple states
        self.get_labelling(subproperty_index)
            .get(&state_index)
            .expect("Should contain state labelling")
    }

    pub fn get_state_root_label(&self, state_index: StateId) -> &Label {
        self.get_state_label(0, state_index)
    }

    pub fn get_labelling(&self, subproperty_index: usize) -> &BTreeMap<StateId, Label> {
        &self
            .check_map
            .get(&subproperty_index)
            .expect("Labelling should be present")
            .labelling
    }
}
