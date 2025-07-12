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
    history: BTreeMap<HistoryIndex, HistoryPoint>,
}

impl Label {
    fn constant(value: ThreeValued) -> Self {
        let history_index = HistoryIndex(0);
        let history_point = HistoryPoint {
            value,
            next_states: vec![],
        };
        let mut history = BTreeMap::new();
        history.insert(history_index, history_point);

        Self { history }
    }
}

impl Label {
    fn last_point(&self) -> &HistoryPoint {
        self.history
            .last_key_value()
            .map(|(_key, value)| value)
            .expect("History point should have last value")
    }

    fn at_history_index(&self, history_index: &HistoryIndex) -> &HistoryPoint {
        self.at_history_index_key_value(history_index).1
    }

    fn at_history_index_key_value(
        &self,
        history_index: &HistoryIndex,
    ) -> (&HistoryIndex, &HistoryPoint) {
        self.history
            .range(..=history_index)
            .last()
            .expect("History point should have a value at or before given history index")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct HistoryIndex(u64);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct HistoryPoint {
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
            .map(|(state_id, label)| (*state_id, label.last_point().value))
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
        new_states: &BTreeSet<StateId>,
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
                //let subproperty = property.subproperty_entry(*subproperty_id);
                //if matches!(subproperty.ty, PropertyType::Next(_)) {
                /*check_info.dirty.extend(changed_successors);
                for state_id in changed_successors {
                    check_info.reasons.remove(state_id);
                }*/
                //check_info.dirty.clear();
                //check_info.labelling.clear();
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
            let label = labelling
                .get(&initial_state_id)
                .expect("Labelling should contain initial state");
            let state_value = label.last_point().value;

            result = result & state_value;
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
                update = self.compute_labelling(property, space, *inner)?;
                for (_state_id, label_update) in update.iter_mut() {
                    for history_point in label_update.history.values_mut() {
                        history_point.value = !history_point.value;
                    }
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
                    let mut variable_label = fixed_point_labelling.get(&state_id).expect(
                        "Fixed-point variable computation should have state labelling available",
                    ).clone();

                    for history_point in variable_label.history.values_mut() {
                        history_point.next_states = Vec::new();
                    }

                    update.insert(state_id, variable_label);
                }
            }
        };

        let check_info = self.get_check_info_mut(property, subproperty_index);

        let updated_states = Self::update_labelling(check_info, update);

        if log_enabled!(log::Level::Trace) {
            trace!(
                "Computed subproperty {:?} update {:#?}",
                subproperty_entry,
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
    ) -> BTreeMap<StateId, Label> {
        let mut updated_labels = BTreeMap::new();

        for (state_id, new_label) in update {
            if let Some(current_label) = check_info.labelling.get_mut(&state_id) {
                if new_label == *current_label {
                    continue;
                }

                *current_label = new_label.clone();
            } else {
                check_info.labelling.insert(state_id, new_label.clone());
            }
            updated_labels.insert(state_id, new_label);
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

            for history_index in history_indices {
                let a_point = a_label.at_history_index(&history_index);
                let b_point = b_label.at_history_index(&history_index);

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

                result_history.insert(history_index, result_point.clone());
            }

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
        mut dirty: BTreeSet<StateId>,
        update: &mut BTreeMap<StateId, Label>,
        op: &NextOperator,
    ) -> Result<(), ExecError> {
        let ground_value = ThreeValued::from_bool(op.is_universal);

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
            let mut history_indices = BTreeSet::new();

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

            for successor_label in direct_successors.values() {
                for history_index in successor_label.history.keys() {
                    if !history_indices.contains(history_index) {
                        history_indices.insert(*history_index);
                    }
                }
            }

            let mut result_history = BTreeMap::new();

            //println!("Computing next for history indices {:?}", history_indices);

            for history_index in history_indices {
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
                        successor_label.at_history_index_key_value(&history_index);
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
                result_history.insert(history_index, history_point);
            }

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
        fixed_point_index: usize,
        mut dirty: BTreeSet<StateId>,
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

        // make sure that all dirty states are set to ground labelling at first within our history index so they can be completely recomputed

        let ground_history_point = HistoryPoint {
            next_states: Vec::new(),
            value: ground_value,
        };

        // TODO: figure out the history index for the variable reset more elegantly

        let mut ground_history_index = HistoryIndex(0);

        for state_id in space.states() {
            if let Some(label) = check_info.labelling.get(&state_id) {
                if let Some((history_index, _value)) = label.history.last_key_value() {
                    ground_history_index =
                        HistoryIndex(ground_history_index.0.max(history_index.0 + 1));
                }
            }
        }

        // TODO: do not do a ground update for all states, but only on an as-needed basis
        dirty = BTreeSet::from_iter(space.states());

        let ground_update = BTreeMap::new();

        for state_id in dirty {
            let label = check_info
                .labelling
                .entry(state_id)
                .or_insert_with(|| Label {
                    history: BTreeMap::new(),
                });
            label
                .history
                .insert(ground_history_index, ground_history_point.clone());
        }

        Self::update_labelling(check_info, ground_update);

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

            for (state_id, label) in &mut current_update {
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
                }
            }

            // update the labelling and make updated dirty in the variable

            let updated = Self::update_labelling(fixed_point_info, current_update);

            /*println!(
                "{:?}: computed fixed point, lowest updated index: {:?}, updated {:#?}",
                history_index, lowest_updated_index, updated
            );*/

            if updated.is_empty() {
                // fixed-point reached
                // the labelling now definitely corresponds to inner
                // just clear dirty as everything was computed

                //println!("Reached fixed point at {:?}", history_index);

                fixed_point_info.dirty.clear();
                return Ok(all_updated);
            };

            fixed_point_info.dirty = BTreeSet::from_iter(updated.keys().copied());

            all_updated.extend(updated);

            //println!("Really changed: {:?}", updated);
        }
    }

    /*fn update_fixed_point_labelling(
        check_info: &mut CheckInfo,
        update: BTreeMap<StateId, Label>,
    ) -> (BTreeMap<StateId, Label>, Option<HistoryIndex>) {
        let mut lowest_updated_index = None;

        let mut updated_labels = BTreeMap::new();

        for (state_id, new_label) in update {
            if let Some(current_label) = check_info.labelling.get_mut(&state_id) {
                if new_label == *current_label {
                    continue;
                }

                let mut history_indices = BTreeSet::new();
                for key in new_label.history.keys().chain(current_label.history.keys()) {
                    if !history_indices.contains(key) {
                        history_indices.insert(*key);
                    }
                }

                for history_index in history_indices {
                    let current_point = current_label.history.get(&history_index);
                    let new_point = new_label.history.get(&history_index);
                    if let (Some(current_point), Some(new_point)) = (current_point, new_point) {
                        if current_point == new_point {
                            continue;
                        }
                    }
                    lowest_updated_index = Some(history_index);
                    break;
                }

                *current_label = new_label.clone();
            } else {
                if let Some((history_index, _history_point)) = new_label.history.first_key_value() {
                    if lowest_updated_index
                        .as_ref()
                        .is_none_or(|lowest_updated_index| lowest_updated_index < history_index)
                    {
                        lowest_updated_index = Some(history_index.clone());
                    }
                }
                check_info.labelling.insert(state_id, new_label.clone());
            }
            updated_labels.insert(state_id, new_label);
        }

        (updated_labels, lowest_updated_index)
    }*/

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
