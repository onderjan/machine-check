mod deduce;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use log::{log_enabled, trace};
use machine_check_common::{
    check::{Conclusion, PreparedProperty},
    property::{BiOperator, FixedPointOperator, Property, TemporalOperator, UniOperator},
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
    property: &PreparedProperty,
) -> Result<Conclusion, ExecError> {
    let property = property.canonical();
    let mut checker = ThreeValuedChecker::new(space);
    checker.check_property(property)
}

pub(super) fn check_property_with_labelling<M: FullMachine>(
    space: &StateSpace<M>,
    property: &PreparedProperty,
) -> Result<(Conclusion, BTreeMap<StateId, ThreeValued>), ExecError> {
    let property = property.canonical();
    let mut checker = ThreeValuedChecker::new(space);
    let conclusion = checker.check_property(property)?;

    // get the labelling as well
    let (property_id, _updated) = checker.compute_labelling(property)?;
    let labelling = checker.get_labelling(property_id).clone();
    Ok((conclusion, labelling))
}

/// Three-valued model checker.
struct ThreeValuedChecker<'a, M: FullMachine> {
    space: &'a StateSpace<M>,
    property_ids: HashMap<Property, PropertyId>,
    check_map: HashMap<PropertyId, CheckInfo>,
    next_property_id: u64,
}

struct CheckInfo {
    labelling: BTreeMap<StateId, ThreeValued>,
    reasons: BTreeMap<StateId, StateId>,
    dirty: BTreeSet<StateId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PropertyId(u64);

impl<'a, M: FullMachine> ThreeValuedChecker<'a, M> {
    fn new(space: &'a StateSpace<M>) -> Self {
        Self {
            space,
            check_map: HashMap::new(),
            property_ids: HashMap::new(),
            next_property_id: 0,
        }
    }

    /// Model-checks a CTL proposition.
    ///
    /// The proposition must be prepared and made canonical beforehand.
    fn check_property(&mut self, property: &Property) -> Result<Conclusion, ExecError> {
        if !self.space.is_valid() {
            return Ok(Conclusion::NotCheckable);
        }

        // compute optimistic and pessimistic interpretation and get the conclusion from that
        match self.compute_interpretation(property)? {
            ThreeValued::False => Ok(Conclusion::Known(false)),
            ThreeValued::True => Ok(Conclusion::Known(true)),
            ThreeValued::Unknown => Ok(Conclusion::Unknown(deduce_culprit(self, property)?)),
        }
    }

    fn compute_interpretation(&mut self, prop: &Property) -> Result<ThreeValued, ExecError> {
        let (property_id, _updated) = self.compute_labelling(prop)?;
        let labelling = self.get_labelling(property_id);
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
        prop: &Property,
    ) -> Result<(PropertyId, BTreeSet<StateId>), ExecError> {
        let property_id = self.get_or_insert_property_id(prop);
        if let std::collections::hash_map::Entry::Vacant(e) = self.check_map.entry(property_id) {
            // make all states dirty by default
            e.insert(CheckInfo {
                labelling: BTreeMap::new(),
                reasons: BTreeMap::new(),
                dirty: BTreeSet::from_iter(self.space.states()),
            });
        }

        match prop {
            Property::Const(c) => {
                let check_info = self.get_check_info_mut(property_id);

                let constant = ThreeValued::from_bool(*c);
                // make dirty constant labelling
                for state_id in &check_info.dirty {
                    check_info.labelling.insert(*state_id, constant);
                }
            }
            Property::Atomic(atomic_property) => {
                let check_info = self
                    .check_map
                    .get_mut(&property_id)
                    .expect("Check info for atomic property should be available");
                // get dirty from space
                for state_id in check_info.dirty.iter().copied() {
                    check_info.labelling.insert(
                        state_id,
                        self.space.atomic_label(atomic_property, state_id)?,
                    );
                }
            }
            Property::Negation(inner) => {
                // negate everything dirty or updated
                let (inner_property_id, inner_updated) = self.compute_labelling(&inner.0)?;
                let mut dirty = BTreeSet::new();
                dirty.append(&mut self.get_check_info_mut(property_id).dirty);
                dirty.extend(inner_updated);

                let inner_labelling = self.get_labelling(inner_property_id);

                let mut updated_labelling = BTreeMap::new();

                for state_id in dirty.iter().copied() {
                    let value = !*inner_labelling
                        .get(&state_id)
                        .expect("Negation should have inner state labelling");
                    updated_labelling.insert(state_id, value);
                }

                let check_info = self.get_check_info_mut(property_id);
                check_info.dirty = dirty;
                check_info.labelling.extend(updated_labelling);
            }
            Property::Or(operator) => self.compute_binary_op(property_id, operator, false)?,
            Property::And(operator) => self.compute_binary_op(property_id, operator, true)?,
            Property::E(TemporalOperator::X(inner)) => {
                self.compute_next_labelling(property_id, inner, false)?
            }
            Property::A(TemporalOperator::X(inner)) => {
                self.compute_next_labelling(property_id, inner, true)?
            }
            Property::LeastFixedPoint(operator) => {
                self.compute_fixed_point_op(property_id, operator, ThreeValued::from_bool(false))?
            }
            Property::GreatestFixedPoint(operator) => {
                self.compute_fixed_point_op(property_id, operator, ThreeValued::from_bool(true))?
            }
            Property::FixedPointVariable(_) => {
                // the variable has been initialised / computed within the fixed-point operators
                assert!(self.check_map.contains_key(&property_id));
            }
            _ => panic!("expected {:?} to be canonical", prop),
        };

        let check_info = self.get_check_info_mut(property_id);
        let mut updated_states = BTreeSet::new();
        std::mem::swap(&mut check_info.dirty, &mut updated_states);

        if log_enabled!(log::Level::Trace) {
            trace!(
                "Computed property {} labelling {:?}, updated {:?}",
                prop,
                check_info.labelling,
                updated_states
            );
        }

        Ok((property_id, updated_states))
    }

    fn get_check_info_mut(&mut self, property_id: PropertyId) -> &mut CheckInfo {
        self.check_map
            .get_mut(&property_id)
            .expect("Check info for the property should be available")
    }

    fn compute_binary_op(
        &mut self,
        property_id: PropertyId,
        operator: &BiOperator,
        is_and: bool,
    ) -> Result<(), ExecError> {
        let mut dirty = BTreeSet::new();
        dirty.append(&mut self.get_check_info_mut(property_id).dirty);

        let (a_id, a_updated) = self.compute_labelling(&operator.a)?;
        let (b_id, b_updated) = self.compute_labelling(&operator.b)?;

        let a_labelling = self.get_labelling(a_id);
        let b_labelling = self.get_labelling(b_id);

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

            let result_value = if is_and {
                a_value & b_value
            } else {
                a_value | b_value
            };

            updated_labelling.insert(state_id, result_value);
        }

        let check_info = self.get_check_info_mut(property_id);
        check_info.dirty = dirty;
        check_info.labelling.extend(updated_labelling);

        Ok(())
    }

    fn compute_fixed_point_op(
        &mut self,
        property_id: PropertyId,
        operator: &FixedPointOperator,
        initial_value: ThreeValued,
    ) -> Result<(), ExecError> {
        let constant_labelling = self.constant_labelling(initial_value);

        // initialise variable labelling
        let variable_property = Property::FixedPointVariable(operator.variable.clone());
        let variable_property_id = self.get_or_insert_property_id(&variable_property);

        self.check_map
            .entry(variable_property_id)
            .or_insert_with(|| CheckInfo {
                labelling: constant_labelling,
                reasons: BTreeMap::new(),
                dirty: BTreeSet::from_iter(self.space.states()),
            });

        let mut all_updated = BTreeSet::new();

        //println!("Computing fixed point");

        // compute inner property labelling and update variable labelling until they match
        loop {
            let (inner_property_id, updated) = self.compute_labelling(&operator.inner)?;

            //println!("Updated in this iteration: {:?}", updated);

            all_updated.extend(updated.iter().cloned());

            if updated.is_empty() {
                // fixed-point reached
                let inner_labelling = self
                    .check_map
                    .get(&inner_property_id)
                    .expect("Check map should contain inner property")
                    .labelling
                    .clone();
                let check_map = self
                    .check_map
                    .get_mut(&property_id)
                    .expect("Check map should contain fixed-point property");
                check_map.labelling = inner_labelling;
                check_map.dirty = all_updated;

                return Ok(());
            } else {
                let variable_labelling = &self
                    .check_map
                    .get(&variable_property_id)
                    .expect("Check map should contain inner property")
                    .labelling;

                //println!("Variable labelling: {:?}", variable_labelling);

                let mut updated_labels = BTreeMap::new();
                for state_id in updated {
                    let previous = variable_labelling
                        .get(&state_id)
                        .expect("Variable labelling should contain updated state");
                    let inner_labelling = &self
                        .check_map
                        .get(&inner_property_id)
                        .expect("Check map should contain inner property")
                        .labelling;
                    let current = inner_labelling
                        .get(&state_id)
                        .expect("Inner labelling should contain updated state");
                    if current != previous {
                        updated_labels.insert(state_id, *current);
                    }
                }

                // update the labelling and make updated dirty in the variable
                let variable_info = self
                    .check_map
                    .get_mut(&variable_property_id)
                    .expect("Check map should contain variable property");

                //println!("Really changed: {:?}", updated_labels);

                variable_info.dirty.extend(updated_labels.keys().copied());
                variable_info.labelling.extend(updated_labels);
            }
        }
    }

    fn compute_next_labelling(
        &mut self,
        property_id: PropertyId,
        inner: &UniOperator,
        universal: bool,
    ) -> Result<(), ExecError> {
        let ground_value = ThreeValued::from_bool(universal);

        let check_info = &mut self.get_check_info_mut(property_id);
        let mut current_reasons = BTreeMap::new();
        current_reasons.append(&mut check_info.reasons);

        // TODO: remove clone
        /*let (previous_labelling, mut dirty) = {
            let check_info = self.get_check_info_mut(property_id);
            (check_info.labelling.clone(), check_info.dirty.clone())
        };*/

        let mut dirty = BTreeSet::new();
        dirty.append(&mut check_info.dirty);

        let (inner_property_id, inner_updated) = self.compute_labelling(&inner.0)?;

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

        let check_info = &mut self.get_check_info_mut(property_id);
        let mut previous_dirty_labels = BTreeMap::new();
        for state_id in dirty.iter().copied() {
            previous_dirty_labels.insert(state_id, check_info.labelling.get(&state_id).copied());
        }

        let inner_labelling = self.get_labelling(inner_property_id);

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
                if universal {
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

        let check_info = self.get_check_info_mut(property_id);
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

    fn get_state_labelling(&self, prop: &Property, state_index: StateId) -> ThreeValued {
        let property_id = self
            .get_property_id(prop)
            .expect("Should contain property when getting state labelling");
        *self
            .get_labelling(property_id)
            .get(&state_index)
            .expect("Should contain state labelling")
    }

    fn get_labelling(&self, property_id: PropertyId) -> &BTreeMap<StateId, ThreeValued> {
        &self
            .check_map
            .get(&property_id)
            .expect("Labelling should be present")
            .labelling
    }

    fn get_reasons(&self, property_id: PropertyId) -> &BTreeMap<StateId, StateId> {
        &self
            .check_map
            .get(&property_id)
            .expect("Reasons should be present")
            .reasons
    }

    fn get_property_id(&self, property: &Property) -> Option<PropertyId> {
        self.property_ids.get(property).cloned()
    }

    fn get_or_insert_property_id(&mut self, property: &Property) -> PropertyId {
        if let Some(id) = self.get_property_id(property) {
            return id;
        }

        let id = PropertyId(self.next_property_id);
        self.next_property_id = self
            .next_property_id
            .checked_add(1)
            .expect("Property id should not overflow");
        self.property_ids.insert(property.clone(), id);
        id
    }
}
