use std::collections::{BTreeMap, BTreeSet};

use log::trace;
use mck::abstr::AbstractValue;

use machine_check_common::iir::ISubpropertyFunc;
use machine_check_common::{ExecError, ParamValuation, StateId, ThreeValued};

use crate::model_check::property_checker::labelling_updater::LabellingUpdater;
use crate::model_check::property_checker::{CheckChoice, CheckValue};
use crate::FullMachine;
use crate::MetaWrap;

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub(super) fn update_func(
        &mut self,
        op: &ISubpropertyFunc,
    ) -> Result<BTreeMap<StateId, CheckValue>, ExecError> {
        trace!("Updating function labelling");

        let func = &op.func;

        let mut labellings = BTreeMap::new();
        let mut updated_states = BTreeSet::new();

        for dependency in &op.dependencies {
            let labelling = if op.children.contains(dependency) {
                self.update_labelling(*dependency)?
            } else {
                let mut labelling = BTreeMap::new();
                for state_id in self.property_checker.focus.dirty_iter() {
                    labelling.insert(
                        state_id,
                        self.getter().compute_latest(*dependency, state_id)?,
                    );
                }
                labelling
            };

            updated_states.extend(labelling.keys());
            labellings.insert(dependency, labelling);
        }

        let mut result = BTreeMap::new();

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

                        let value = if let Some(value) = labellings
                            .get_mut(&input_subproperty_index)
                            .expect("Input subproperty should be in labellings")
                            .remove(&state_id)
                        {
                            value
                        } else {
                            self.getter()
                                .compute_latest(input_subproperty_index, state_id)?
                        };

                        let boolean = match value.valuation {
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

            let input_values = input_values.into_iter().map(MetaWrap).collect();

            let AbstractValue::Boolean(result_value) = result_value else {
                panic!("Result should be abstract Boolean");
            };

            let valuation = ParamValuation::from_three_valued(result_value.into_three_valued());

            let state_result = CheckValue {
                valuation,
                choice: CheckChoice::Func(input_values),
            };

            result.insert(state_id, state_result);
        }

        Ok(result)
    }
}
