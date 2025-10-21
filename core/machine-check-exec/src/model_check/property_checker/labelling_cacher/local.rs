use std::collections::BTreeMap;

use mck::abstr::{Abstr, AbstractValue, Manipulatable};

use machine_check_common::iir::property::{ISubproperty, ISubpropertyFunc};
use machine_check_common::{ExecError, ParamValuation, StateId, ThreeValued};

use crate::model_check::property_checker::labelling_cacher::LabellingCacher;
use crate::model_check::property_checker::value::TimedCheckValue;
use crate::model_check::property_checker::{CheckChoice, CheckValue};
use crate::FullMachine;
use crate::MetaWrap;

impl<M: FullMachine> LabellingCacher<'_, M> {
    pub(super) fn compute_func(
        &self,
        op: &ISubpropertyFunc,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        self.apply_func(op, state_id, &mut BTreeMap::new())
    }

    pub fn apply_func(
        &self,
        op: &ISubpropertyFunc,
        state_id: StateId,
        labellings: &mut BTreeMap<usize, BTreeMap<StateId, TimedCheckValue>>,
    ) -> Result<TimedCheckValue, ExecError> {
        let func = &op.func;

        let mut globals = BTreeMap::new();

        let state_data = self.space.state_data(state_id);

        let state_result = &state_data.result;
        let state_panic = &state_data.panic;

        let mut input_choices = Vec::new();

        let mut max_unknown_time = 0;

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

                let timed = if let Some(timed) = labellings
                    .get_mut(&input_subproperty_index)
                    .and_then(|labelling| labelling.remove(&state_id))
                {
                    timed
                } else {
                    let mut timed = None;
                    if !op.children.contains(&input_subproperty_index)
                        && matches!(
                            self.property_checker.property.subproperties[input_subproperty_index],
                            ISubproperty::FixedPoint(_)
                        )
                    {
                        // we should have the variable instead of computing the fixed point
                        timed =
                            Some(self.compute_fixed_variable(input_subproperty_index, state_id)?);
                    }

                    if let Some(timed) = timed {
                        timed
                    } else {
                        self.compute_latest_timed(input_subproperty_index, state_id)?
                    }
                };

                let mut choice = None;

                let boolean = match timed.value {
                    CheckValue::False => mck::abstr::Boolean::from_three_valued(ThreeValued::False),
                    CheckValue::True => mck::abstr::Boolean::from_three_valued(ThreeValued::True),
                    CheckValue::Dependent => todo!(),
                    CheckValue::Unknown(timed_choice) => {
                        max_unknown_time = max_unknown_time.max(timed.time);
                        choice = Some(*timed_choice);
                        mck::abstr::Boolean::from_three_valued(ThreeValued::Unknown)
                    }
                };

                let value = AbstractValue::Boolean(boolean);

                input_choices.push((MetaWrap(value.clone()), choice));

                value
            } else {
                let value = if input_var_name == "__panic" {
                    state_panic.to_runtime()
                } else {
                    let Some(field) = Manipulatable::get(state_result, input_var_name) else {
                        panic!("Input '{}' should be in fields", input_var_name);
                    };

                    field.runtime_value()
                };
                input_choices.push((MetaWrap(value.clone()), None));
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

        let value = match valuation {
            ParamValuation::False => CheckValue::False,
            ParamValuation::True => CheckValue::True,
            ParamValuation::Dependent => CheckValue::Dependent,
            ParamValuation::Unknown => {
                CheckValue::Unknown(Box::new(CheckChoice::Func(input_choices)))
            }
        };

        Ok(TimedCheckValue {
            time: max_unknown_time,
            value,
        })
    }
}
