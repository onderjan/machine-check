use std::collections::BTreeMap;

use machine_check_common::{
    iir::{
        context::IContext,
        description::IMachine,
        func::IFn,
        path::{IIdent, ISpan},
        property::{IProperty, ISubproperty},
    },
    ExecError, NodeId, ParamValuation, StateId, ThreeValued,
};
use mck::{
    abstr::{Abstr, AbstractValue, BitvectorDomain},
    concr::FullMachine,
};

use crate::space::StateSpace;

/// Checks the property non-incrementally.
///
/// This is supposed to be a simple algorithm with basic computation
/// that will run only when incremental property checking determines
/// a known value. This provides an additional sanity check that
/// the incremental model-checking really produced a correct result.
pub fn check_property<M: FullMachine>(
    space: &StateSpace<M>,
    machine: &IMachine,
    property: &IProperty,
) -> Result<ParamValuation, ExecError> {
    let mut environment = BTreeMap::new();
    NonincrementalChecker {
        space,
        machine,
        property,
        environment: &mut environment,
    }
    .check_property()
}

pub(super) struct NonincrementalChecker<'a, M: FullMachine> {
    pub(super) space: &'a StateSpace<M>,
    pub(super) machine: &'a IMachine,
    pub(super) property: &'a IProperty,
    pub(super) environment: &'a mut BTreeMap<(usize, StateId), ParamValuation>,
}

impl<M: FullMachine> NonincrementalChecker<'_, M> {
    pub(super) fn check_property(&mut self) -> Result<ParamValuation, ExecError> {
        self.environment.clear();

        self.check_subproperty(0)?;

        // treat as AX! from root node
        Ok(self.compute_next_value(true, 0, NodeId::ROOT))
    }

    fn check_subproperty(&mut self, subproperty_index: usize) -> Result<(), ExecError> {
        let subproperty_entry = &self.property.subproperties[subproperty_index];

        match subproperty_entry {
            ISubproperty::Func(subproperty_func) => {
                for dependency in &subproperty_func.children {
                    self.check_subproperty(*dependency)?;
                }

                for state_id in self.space.states() {
                    let value = self.compute_fn_value(&subproperty_func.func, state_id);
                    self.environment
                        .insert((subproperty_index, state_id), value);
                }
            }
            ISubproperty::Next(next) => {
                self.check_subproperty(next.inner)?;
                for state_id in self.space.states() {
                    let value =
                        self.compute_next_value(next.universal, next.inner, state_id.into());
                    self.environment
                        .insert((subproperty_index, state_id), value);
                }
            }
            ISubproperty::FixedPoint(fixed_point) => {
                self.check_fixed_point(
                    subproperty_index,
                    fixed_point.universal,
                    fixed_point.inner,
                )?;
            }
        }

        if log::log_enabled!(log::Level::Trace) {
            let subprop_env: BTreeMap<StateId, ParamValuation> = self
                .environment
                .iter()
                .filter(|((index, _), _)| *index == subproperty_index)
                .map(|((_, state_id), value)| (*state_id, *value))
                .collect();

            log::trace!(
                "Resolved subproperty #{} environment:\n{:?}",
                subproperty_index,
                subprop_env,
            );
        }
        Ok(())
    }

    fn check_fixed_point(
        &mut self,
        subproperty_index: usize,
        is_greatest: bool,
        inner_index: usize,
    ) -> Result<(), ExecError> {
        // set fixed-point values to ground value (true for universal, false for existential)
        let ground_value = ParamValuation::from_bool(is_greatest);
        for state_id in self.space.states() {
            self.environment
                .insert((subproperty_index, state_id), ground_value);
        }

        loop {
            // check inner
            self.check_subproperty(inner_index)?;

            // update the fixed-point values with inner
            let mut updated = false;
            for state_id in self.space.states() {
                let previous_value = *self
                    .environment
                    .get(&(subproperty_index, state_id))
                    .expect("Previous value should be present");
                let current_value = *self
                    .environment
                    .get(&(inner_index, state_id))
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

    fn compute_fn_value(&self, func: &IFn, state_id: StateId) -> ParamValuation {
        let mut globals = BTreeMap::new();

        let state_data = self.space.state_data(state_id);

        let state_result = &state_data.result;
        let state_panic = &state_data.panic;

        for input_var_id in &func.signature.inputs {
            let input_var_name = func
                .variables
                .get(input_var_id)
                .expect("Input should be in variables")
                .ident
                .name();
            let value = if let Some(stripped) = input_var_name.strip_prefix("__mck_subproperty_") {
                let Ok(input_subproperty_index) = stripped.parse::<usize>() else {
                    panic!("Input subproperty should have valid index");
                };

                let valuation = *self
                    .environment
                    .get(&(input_subproperty_index, state_id))
                    .expect("Input valuation should be present");

                let boolean = match valuation {
                    ParamValuation::False => {
                        mck::abstr::Boolean::from_three_valued(ThreeValued::False)
                    }
                    ParamValuation::True => {
                        mck::abstr::Boolean::from_three_valued(ThreeValued::True)
                    }
                    ParamValuation::Dependent => todo!(),
                    ParamValuation::Unknown => {
                        mck::abstr::Boolean::from_three_valued(ThreeValued::Unknown)
                    }
                };

                AbstractValue::Boolean(boolean)
            } else if input_var_name == "__panic" {
                state_panic.to_runtime()
            } else {
                let state_fields = &self.machine.state().fields;
                let AbstractValue::Struct(state_field_values) = state_result.to_runtime() else {
                    panic!("Input '{}' should be in a struct", input_var_name);
                };

                assert_eq!(state_fields.len(), state_field_values.len());

                let Some(field_index) = state_fields
                    .get_index_of(&IIdent::new(input_var_name.to_string(), ISpan::Unspecified))
                else {
                    panic!("Input '{}' should be in fields", input_var_name);
                };

                state_field_values[field_index].clone()
            };

            globals.insert(input_var_name.to_string(), value);
        }

        let input_values = func.globals_to_input_values(&globals);

        let (result, panic) = func.call(&IContext::empty(), input_values);

        // TODO: raise an error on nonzero panic result
        assert!(panic
            .expect_bitvector()
            .concrete_value()
            .is_some_and(|v| v.is_zero()));

        let AbstractValue::Boolean(result) = result else {
            panic!("Result should be abstract Boolean");
        };

        ParamValuation::from_three_valued(result.into_three_valued())
    }

    fn compute_next_value(
        &self,
        is_universal: bool,
        inner: usize,
        node_id: NodeId,
    ) -> ParamValuation {
        let param_partition = self
            .space
            .direct_successor_param_partition(node_id)
            .expect("Each state should have at least one successor");

        // compute for each parameter separately and then put them together
        let mut can_be_unknown = false;
        let mut can_be_false = false;
        let mut can_be_true = false;

        for param_set in param_partition.all_sets() {
            let mut parameter_value = ParamValuation::from_bool(is_universal);
            for successor_id in param_set.map(|(_, state_id)| *state_id) {
                let successor_value = *self
                    .environment
                    .get(&(inner, successor_id))
                    .expect("Left value should be present");

                parameter_value =
                    Self::choose_binary(is_universal, parameter_value, successor_value);
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
    }
}
