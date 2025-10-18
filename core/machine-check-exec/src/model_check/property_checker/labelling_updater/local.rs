use std::collections::{BTreeMap, BTreeSet};

use log::trace;
use mck::abstr::AbstractValue;

use machine_check_common::iir::ISubpropertyFunc;
use machine_check_common::{ExecError, ParamValuation, StateId, ThreeValued};

use crate::model_check::property_checker::labelling_updater::LabellingUpdater;
use crate::model_check::property_checker::value::TimedCheckValue;
use crate::model_check::property_checker::{CheckChoice, CheckValue};
use crate::FullMachine;
use crate::MetaWrap;

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub(super) fn update_func(
        &mut self,
        op: &ISubpropertyFunc,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        trace!("Updating function labelling");

        let func = &op.func;

        let mut labellings = BTreeMap::new();
        let mut updated_states = BTreeSet::new();

        for dependency in &op.dependencies {
            let labelling = self.update_labelling(*dependency)?;
            updated_states.extend(labelling.keys());
            labellings.insert(dependency, labelling);
        }

        let mut result = BTreeMap::new();

        let mut last_time = 0;

        for state_id in updated_states {
            let state_data = self.space.state_data(state_id);
            let state_result = &state_data.result;
            let state_panic = &state_data.panic;

            let mut globals = BTreeMap::new();
            for input_var_id in &func.signature.inputs {
                let input_var_name = func
                    .variables
                    .get(input_var_id)
                    .expect("Input should be in variables")
                    .ident
                    .name();
                let value =
                    if let Some(stripped) = input_var_name.strip_prefix("__mck_subproperty_") {
                        let Ok(input_subproperty_index) = stripped.parse::<usize>() else {
                            panic!("Input subproperty should have valid index");
                        };

                        let timed = if let Some(timed) = labellings
                            .get_mut(&input_subproperty_index)
                            .expect("Input subproperty should be in labellings")
                            .remove(&state_id)
                        {
                            timed
                        } else {
                            self.getter()
                                .compute_latest_timed(input_subproperty_index, state_id)?
                        };

                        last_time = last_time.max(timed.time);
                        let valuation = timed.value.valuation;

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
                        AbstractValue::Bitvector(state_panic.to_runtime())
                    } else {
                        use mck::abstr::Manipulatable;

                        let Some(field) = state_result.get(input_var_name) else {
                            panic!("Input '{}' should be in fields", input_var_name);
                        };

                        field.runtime_value()
                    };

                globals.insert(input_var_name.to_string(), value);
            }

            let input_values = func.globals_to_input_values(&globals);

            let result_value = func.call(input_values.clone());

            let input_values = input_values.into_iter().map(|e| MetaWrap(e)).collect();

            let AbstractValue::Boolean(result_value) = result_value else {
                panic!("Result should be abstract Boolean");
            };

            let valuation = ParamValuation::from_three_valued(result_value.into_three_valued());

            let state_result = TimedCheckValue {
                time: last_time,
                value: CheckValue {
                    valuation,
                    choice: CheckChoice::Func(input_values),
                },
            };

            result.insert(state_id, state_result);
        }

        Ok(result)
    }

    /*pub(super) fn update_negation(
        &mut self,
        inner: usize,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        let inner_result = self.update_labelling(inner)?;
        let mut result = BTreeMap::new();

        for (state_id, timed) in inner_result {
            let timed = LabellingCacher::<M>::apply_negation(timed);

            result.insert(state_id, timed);
        }

        Ok(result)
    }

    pub(super) fn update_binary_op(
        &mut self,
        op: &BiLogicOperator,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        let mut result = self.update_labelling(op.a)?;
        let mut result_b = self.update_labelling(op.b)?;

        for (state_id, timed) in result.iter_mut() {
            let timed_b = if let Some(timed_b) = result_b.remove(state_id) {
                timed_b
            } else {
                self.getter().compute_latest_timed(op.b, *state_id)?
            };

            let choice = LabellingCacher::<M>::choose_binary_op(op, timed, &timed_b);

            if matches!(choice, BiChoice::Right) {
                *timed = timed_b;
            };

            // add the choice
            if let CheckValue::Unknown(choices) = &mut timed.value {
                choices.push(CheckChoice::BiLogic(choice));
            };
        }

        for (state_id, timed_b) in result_b {
            let timed_a = self.getter().compute_latest_timed(op.a, state_id)?;

            let choice = LabellingCacher::<M>::choose_binary_op(op, &timed_a, &timed_b);
            let mut timed = match choice {
                BiChoice::Left => timed_a,
                BiChoice::Right => timed_b,
            };

            // add the choice
            if let CheckValue::Unknown(choices) = &mut timed.value {
                choices.push(CheckChoice::BiLogic(choice));
            };

            result.insert(state_id, timed);
        }

        Ok(result)
    }*/
}
