use std::collections::BTreeMap;

use machine_check_common::{
    iir::{func::IFn, interpretation::IAbstractValue, IProperty},
    ExecError, NodeId, ParamValuation, StateId, ThreeValued,
};
use mck::{abstr::Manipulatable, concr::FullMachine};

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
    let mut environment = BTreeMap::new();
    IncrementalChecker {
        space,
        property,
        environment: &mut environment,
        //calmable_fixed_points: BTreeSet::new(),
    }
    .check_property()
}

#[derive(Clone, Debug)]
pub enum CheckChoice {
    Next(Option<StateId>),
    FixedPoint,
    Func(Vec<IAbstractValue>),
}

#[derive(Clone, Debug)]
pub struct CheckValue {
    pub valuation: ParamValuation,
    pub choice: CheckChoice,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BiChoice {
    Left,
    Right,
}

pub(super) struct IncrementalChecker<'a, M: FullMachine> {
    pub(super) space: &'a StateSpace<M>,
    pub(super) property: &'a IProperty,
    pub(super) environment: &'a mut BTreeMap<(usize, StateId), CheckValue>,
    //calmable_fixed_points: BTreeSet<usize>,
}

impl<M: FullMachine> IncrementalChecker<'_, M> {
    pub(super) fn check_property(&mut self) -> Result<ParamValuation, ExecError> {
        self.environment.clear();

        self.check_subproperty(0)?;

        // treat as AX! from root node
        Ok(self.compute_next_value(true, 0, NodeId::ROOT).valuation)
    }

    fn update_value(&mut self, subproperty_index: usize, state_id: StateId, value: CheckValue) {
        let should_update =
            if let Some(previous_value) = self.environment.get(&(subproperty_index, state_id)) {
                value.valuation != previous_value.valuation
            } else {
                true
            };

        if should_update {
            self.environment
                .insert((subproperty_index, state_id), value);
        }
    }

    fn check_subproperty(&mut self, subproperty_index: usize) -> Result<(), ExecError> {
        let subproperty_entry = &self.property.subproperties[subproperty_index];

        match subproperty_entry {
            machine_check_common::iir::ISubproperty::Func(func, children) => {
                for child in children {
                    self.check_subproperty(*child)?;
                }

                for state_id in self.space.states() {
                    let value = self.compute_fn_value(func, state_id);
                    self.update_value(subproperty_index, state_id, value);
                }
            }
            machine_check_common::iir::ISubproperty::Next(next) => {
                self.check_subproperty(next.inner)?;
                for state_id in self.space.states() {
                    let value =
                        self.compute_next_value(next.universal, next.inner, state_id.into());
                    self.update_value(subproperty_index, state_id, value);
                }
            }
            machine_check_common::iir::ISubproperty::FixedPoint(fixed_point) => {
                self.check_fixed_point(
                    subproperty_index,
                    fixed_point.universal,
                    fixed_point.inner,
                )?;
            }
        }

        if log::log_enabled!(log::Level::Trace) {
            let subprop_env: BTreeMap<StateId, &CheckValue> = self
                .environment
                .iter()
                .filter(|((index, _), _)| *index == subproperty_index)
                .map(|((_, state_id), value)| (*state_id, value))
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
            self.environment.insert(
                (subproperty_index, state_id),
                CheckValue {
                    valuation: ground_value,
                    choice: CheckChoice::FixedPoint,
                },
            );
        }

        loop {
            // check inner
            self.check_subproperty(inner_index)?;

            // update the fixed-point values with inner
            let mut updated = false;
            for state_id in self.space.states() {
                let previous_value = self
                    .environment
                    .get(&(subproperty_index, state_id))
                    .expect("Previous value should be present");
                let current_value = self
                    .environment
                    .get(&(inner_index, state_id))
                    .expect("Current value should be present");

                if previous_value.valuation != current_value.valuation {
                    self.environment.insert(
                        (subproperty_index, state_id),
                        CheckValue {
                            valuation: current_value.valuation,
                            choice: CheckChoice::FixedPoint,
                        },
                    );
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

    fn compute_fn_value(&self, func: &IFn, state_id: StateId) -> CheckValue {
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

                let valuation = self
                    .environment
                    .get(&(input_subproperty_index, state_id))
                    .expect("Input valuation should be present")
                    .valuation;

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

                IAbstractValue::Bool(boolean)
            } else if input_var_name == "__panic" {
                IAbstractValue::Bitvector(state_panic.to_runtime())
            } else {
                let Some(field) = state_result.get(input_var_name) else {
                    panic!("Input '{}' should be in fields", input_var_name);
                };

                let bitvec = field
                    .runtime_bitvector()
                    .expect("Input should be a bitvector");
                IAbstractValue::Bitvector(bitvec)
            };

            globals.insert(input_var_name.to_string(), value);
        }

        let input_values = func.globals_to_input_values(&globals);

        let result = func.call(input_values.clone());

        let IAbstractValue::Bool(result) = result else {
            panic!("Result should be abstract Boolean");
        };

        let valuation = ParamValuation::from_three_valued(result.into_three_valued());

        CheckValue {
            valuation,
            choice: CheckChoice::Func(input_values),
        }
    }

    fn compute_next_value(&self, is_universal: bool, inner: usize, node_id: NodeId) -> CheckValue {
        let param_partition = self
            .space
            .direct_successor_param_partition(node_id)
            .expect("Each state should have at least one successor");

        // compute for each parameter separately and then put them together
        let mut can_be_unknown = false;
        let mut can_be_false = false;
        let mut can_be_true = false;

        let mut successor_state_id = None;

        for param_set in param_partition.all_sets() {
            let mut parameter_value = ParamValuation::from_bool(is_universal);
            for successor_id in param_set.map(|(_, state_id)| *state_id) {
                let successor_value = self
                    .environment
                    .get(&(inner, successor_id))
                    .expect("Left value should be present")
                    .valuation;
                let binary_choice =
                    Self::choose_binary(is_universal, parameter_value, successor_value);

                match binary_choice {
                    BiChoice::Left => {}
                    BiChoice::Right => {
                        successor_state_id = Some(successor_id);
                        parameter_value = successor_value;
                    }
                }
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

        let valuation = match (can_be_unknown, can_be_false, can_be_true) {
            (_, true, true) => ParamValuation::Dependent,
            (true, _, _) => ParamValuation::Unknown,
            (false, true, false) => ParamValuation::False,
            (false, false, true) => ParamValuation::True,
            (false, false, false) => {
                panic!("Parameters should be unknown, false, or true")
            }
        };

        let choice = if let ParamValuation::Unknown = valuation {
            Some(successor_state_id.expect("Unknown should be due to state"))
        } else {
            None
        };

        CheckValue {
            valuation,
            choice: CheckChoice::Next(choice),
        }
    }

    fn choose_binary(is_and: bool, left: ParamValuation, right: ParamValuation) -> BiChoice {
        if is_and {
            match (left, right) {
                (ParamValuation::False, _) => BiChoice::Left,
                (_, ParamValuation::False) => BiChoice::Right,

                (ParamValuation::Unknown, _) => BiChoice::Left,
                (_, ParamValuation::Unknown) => BiChoice::Right,
                (ParamValuation::Dependent, _) => BiChoice::Left,
                (_, ParamValuation::Dependent) => BiChoice::Right,
                (ParamValuation::True, ParamValuation::True) => BiChoice::Left,
            }
        } else {
            match (left, right) {
                (ParamValuation::True, _) => BiChoice::Left,
                (_, ParamValuation::True) => BiChoice::Right,
                (ParamValuation::Unknown, _) => BiChoice::Left,
                (_, ParamValuation::Unknown) => BiChoice::Right,
                (ParamValuation::Dependent, _) => BiChoice::Left,
                (_, ParamValuation::Dependent) => BiChoice::Right,
                (ParamValuation::False, ParamValuation::False) => BiChoice::Left,
            }
        }
    }
}
