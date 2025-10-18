use std::collections::BTreeMap;

use mck::abstr::AbstractValue;

use machine_check_common::iir::ISubpropertyFunc;
use machine_check_common::{ExecError, ParamValuation, StateId, ThreeValued};

use crate::model_check::property_checker::labelling_cacher::LabellingCacher;
use crate::model_check::property_checker::{CheckChoice, CheckValue};
use crate::FullMachine;
use crate::MetaWrap;

impl<M: FullMachine> LabellingCacher<'_, M> {
    pub(super) fn compute_func(
        &self,
        op: &ISubpropertyFunc,
        state_id: StateId,
    ) -> Result<CheckValue, ExecError> {
        self.apply_func(op, state_id, &mut BTreeMap::new())
    }

    pub fn apply_func(
        &self,
        op: &ISubpropertyFunc,
        state_id: StateId,
        labellings: &mut BTreeMap<usize, BTreeMap<StateId, CheckValue>>,
    ) -> Result<CheckValue, ExecError> {
        let func = &op.func;

        let mut globals = BTreeMap::new();

        let state_data = self.space.state_data(state_id);

        let state_result = &state_data.result;
        let state_panic = &state_data.panic;

        let mut input_choices = Vec::new();

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

                let value = if let Some(value) = labellings
                    .get_mut(&input_subproperty_index)
                    .expect("Input subproperty should be in labellings")
                    .remove(&state_id)
                {
                    value
                } else {
                    self.compute_latest(input_subproperty_index, state_id)?
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

                let choice = value.choice;

                let value = AbstractValue::Boolean(boolean);

                input_choices.push((MetaWrap(value.clone()), choice));

                value
            } else if input_var_name == "__panic" {
                let value = AbstractValue::Bitvector(state_panic.to_runtime());
                input_choices.push((
                    MetaWrap(value.clone()),
                    CheckChoice::Atomic(MetaWrap(value.clone())),
                ));
                value
            } else {
                use mck::abstr::Manipulatable;

                let Some(field) = state_result.get(input_var_name) else {
                    panic!("Input '{}' should be in fields", input_var_name);
                };

                let value = field.runtime_value();
                input_choices.push((
                    MetaWrap(value.clone()),
                    CheckChoice::Atomic(MetaWrap(value.clone())),
                ));
                value
            };

            globals.insert(input_var_name.to_string(), value);
        }

        let input_values = func.globals_to_input_values(&globals);

        let result = func.call(input_values.clone());

        let AbstractValue::Boolean(result) = result else {
            panic!("Result should be abstract Boolean");
        };

        let valuation = ParamValuation::from_three_valued(result.into_three_valued());

        Ok(CheckValue {
            valuation,
            choice: CheckChoice::Func(input_choices),
        })
    }
}
