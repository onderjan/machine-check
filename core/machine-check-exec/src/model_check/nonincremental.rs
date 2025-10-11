use std::collections::{BTreeMap, BTreeSet};

use machine_check_common::{iir::IProperty, ExecError, NodeId, ParamValuation, StateId};
use mck::concr::FullMachine;

use crate::space::StateSpace;

/// Checks the property non-incrementally.
///
/// This is supposed to be a simple algorithm with basic computation
/// that will run only when incremental property checking determines
/// a known value. This provides an additional sanity check that
/// the incremental model-checking really produced a correct result.
pub fn check_property<M: FullMachine>(
    space: &StateSpace<M>,
    property: &IProperty,
) -> Result<ParamValuation, ExecError> {
    NonincrementalChecker {
        space,
        property,
        environment: BTreeMap::new(),
        calmable_fixed_points: BTreeSet::new(),
    }
    .check_property()
}

struct NonincrementalChecker<'a, M: FullMachine> {
    space: &'a StateSpace<M>,
    property: &'a IProperty,
    environment: BTreeMap<(usize, StateId), ParamValuation>,
    calmable_fixed_points: BTreeSet<usize>,
}

impl<M: FullMachine> NonincrementalChecker<'_, M> {
    fn check_property(&mut self) -> Result<ParamValuation, ExecError> {
        self.check_subproperty(0)?;

        todo!();

        // treat as AX! from root node
        /*Ok(self.compute_next_value(
            &NextOperator {
                is_universal: true,
                inner: 0,
            },
            NodeId::ROOT,
        ))*/
    }

    fn check_subproperty(&mut self, subproperty_index: usize) -> Result<(), ExecError> {
        let subproperty_entry = self.property.subproperty_entry(subproperty_index);

        todo!("Nonincremental checking");

        /*match &subproperty_entry.ty {
            machine_check_common::property::PropertyType::Const(value) => {
                for state_id in self.space.states() {
                    self.environment.insert(
                        (subproperty_index, state_id),
                        ParamValuation::from_bool(*value),
                    );
                }
            }
            machine_check_common::property::PropertyType::Atomic(atomic_property) => {
                for state_id in self.space.states() {
                    let value = self.space.atomic_label(atomic_property, state_id)?;

                    self.environment.insert(
                        (subproperty_index, state_id),
                        ParamValuation::from_three_valued(value),
                    );
                }
            }
            machine_check_common::property::PropertyType::Negation(inner) => {
                // check inner
                self.check_subproperty(*inner)?;

                // negate inner
                for state_id in self.space.states() {
                    let inner_value = self
                        .environment
                        .get(&(*inner, state_id))
                        .expect("Negation inner value should be present");
                    let value = match inner_value {
                        ParamValuation::False => ParamValuation::True,
                        ParamValuation::True => ParamValuation::False,
                        ParamValuation::Dependent => ParamValuation::Dependent,
                        ParamValuation::Unknown => ParamValuation::Unknown,
                    };
                    self.environment
                        .insert((subproperty_index, state_id), value);
                }
            }
            machine_check_common::property::PropertyType::BiLogic(bi_logic_operator) => {
                let left_index = bi_logic_operator.a;
                let right_index = bi_logic_operator.b;

                // check inner left and right
                self.check_subproperty(left_index)?;
                self.check_subproperty(right_index)?;

                // perform the binary operation
                for state_id in self.space.states() {
                    let left_value = *self
                        .environment
                        .get(&(left_index, state_id))
                        .expect("Left value should be present");
                    let right_value = *self
                        .environment
                        .get(&(right_index, state_id))
                        .expect("Right value should be present");

                    let value =
                        Self::choose_binary(bi_logic_operator.is_and, left_value, right_value);

                    self.environment
                        .insert((subproperty_index, state_id), value);
                }
            }
            machine_check_common::property::PropertyType::Next(next_operator) => {
                // check inner
                self.check_subproperty(next_operator.inner)?;

                // compose the value using the direct successors
                for state_id in self.space.states() {
                    let value = self.compute_next_value(next_operator, state_id.into());

                    self.environment
                        .insert((subproperty_index, state_id), value);
                }
            }
            machine_check_common::property::PropertyType::FixedPoint(fixed_point_operator) => {
                // do not recompute fixed points that have been already computed (calmable)
                // and are closed, i.e. do not contain any fixed-point variables that may change from outside
                // this means the values will not change on a subsequent recomputation, so it is unnecessary
                let is_calm = self.calmable_fixed_points.contains(&subproperty_index)
                    && self.property.is_subproperty_closed_form(subproperty_index);

                if !is_calm {
                    self.check_fixed_point(subproperty_index, fixed_point_operator)?;
                    self.calmable_fixed_points.insert(subproperty_index);
                }
            }
            machine_check_common::property::PropertyType::FixedVariable(fixed_point_index) => {
                // just propagate the value from the fixed point
                for state_id in self.space.states() {
                    let value = self
                        .environment
                        .get(&(*fixed_point_index, state_id))
                        .expect("Fixed-point value should be present");
                    self.environment
                        .insert((subproperty_index, state_id), *value);
                }
            }
        };*/

        if log::log_enabled!(log::Level::Trace) {
            let subprop_env: BTreeMap<StateId, ParamValuation> = self
                .environment
                .iter()
                .filter(|((index, _), _)| *index == subproperty_index)
                .map(|((_, state_id), value)| (*state_id, *value))
                .collect();

            log::trace!(
                "Resolved subproperty #{}: {:?}, environment:\n{:?}",
                subproperty_index,
                subproperty_entry,
                subprop_env,
            );
        }
        Ok(())
    }

    /*fn check_fixed_point(
        &mut self,
        subproperty_index: usize,
        fixed_point_operator: &FixedPointOperator,
    ) -> Result<(), ExecError> {
        // set fixed-point values to ground value (true for universal, false for existential)
        let ground_value = ParamValuation::from_bool(fixed_point_operator.is_greatest);
        for state_id in self.space.states() {
            self.environment
                .insert((subproperty_index, state_id), ground_value);
        }

        loop {
            // check inner
            self.check_subproperty(fixed_point_operator.inner)?;

            // update the fixed-point values with inner
            let mut updated = false;
            for state_id in self.space.states() {
                let previous_value = *self
                    .environment
                    .get(&(subproperty_index, state_id))
                    .expect("Previous value should be present");
                let current_value = *self
                    .environment
                    .get(&(fixed_point_operator.inner, state_id))
                    .expect("Current value should be present");

                if previous_value != current_value {
                    self.environment
                        .insert((subproperty_index, state_id), current_value);
                    updated = true;
                }
            }

            // break if there were no updates
            if !updated {
                break;
            }
        }

        Ok(())
    }

    fn compute_next_value(&self, next_operator: &NextOperator, node_id: NodeId) -> ParamValuation {
        let param_partition = self
            .space
            .direct_successor_param_partition(node_id)
            .expect("Each state should have at least one successor");

        // compute for each parameter separately and then put them together
        let mut can_be_unknown = false;
        let mut can_be_false = false;
        let mut can_be_true = false;

        for param_set in param_partition.all_sets() {
            let mut parameter_value = ParamValuation::from_bool(next_operator.is_universal);
            for successor_id in param_set.map(|(_, state_id)| *state_id) {
                let successor_value = *self
                    .environment
                    .get(&(next_operator.inner, successor_id))
                    .expect("Left value should be present");

                parameter_value = Self::choose_binary(
                    next_operator.is_universal,
                    parameter_value,
                    successor_value,
                );
            }

            match parameter_value {
                ParamValuation::False => can_be_false = true,
                ParamValuation::True => can_be_true = true,
                ParamValuation::Dependent => {
                    can_be_false = true;
                    can_be_true = true;
                }
                ParamValuation::Unknown => can_be_unknown = true,
            }
        }

        match (can_be_unknown, can_be_false, can_be_true) {
            (_, true, true) => ParamValuation::Dependent,
            (true, _, _) => ParamValuation::Unknown,
            (false, true, false) => ParamValuation::False,
            (false, false, true) => ParamValuation::True,
            (false, false, false) => {
                panic!("Parameters should be unknown, false, or true")
            }
        }
    }

    fn choose_binary(is_and: bool, left: ParamValuation, right: ParamValuation) -> ParamValuation {
        if is_and {
            match (left, right) {
                (ParamValuation::False, _) | (_, ParamValuation::False) => ParamValuation::False,
                (ParamValuation::Unknown, _) | (_, ParamValuation::Unknown) => {
                    ParamValuation::Unknown
                }
                (ParamValuation::Dependent, _) | (_, ParamValuation::Dependent) => {
                    ParamValuation::Dependent
                }
                (ParamValuation::True, ParamValuation::True) => ParamValuation::True,
            }
        } else {
            match (left, right) {
                (ParamValuation::True, _) | (_, ParamValuation::True) => ParamValuation::True,
                (ParamValuation::Unknown, _) | (_, ParamValuation::Unknown) => {
                    ParamValuation::Unknown
                }
                (ParamValuation::Dependent, _) | (_, ParamValuation::Dependent) => {
                    ParamValuation::Dependent
                }
                (ParamValuation::False, ParamValuation::False) => ParamValuation::False,
            }
        }
    }*/
}
