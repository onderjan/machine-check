use std::collections::BTreeMap;

use machine_check_common::iir::func::IFn;
use mck::abstr::AbstractValue;

use machine_check_common::iir::{ISubproperty, ISubpropertyFunc};
use machine_check_common::{ExecError, ParamValuation, StateId, ThreeValued};

use crate::model_check::property_checker::labelling_cacher::LabellingCacher;
use crate::model_check::property_checker::value::TimedCheckValue;
use crate::model_check::property_checker::{CheckChoice, CheckValue};
use crate::FullMachine;
use crate::MetaWrap;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum BiChoice {
    Left,
    Right,
}

impl<M: FullMachine> LabellingCacher<'_, M> {
    pub(super) fn compute_func(
        &self,
        op: &ISubpropertyFunc,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        let func = &op.func;

        let mut globals = BTreeMap::new();

        let state_data = self.space.state_data(state_id);

        let state_result = &state_data.result;
        let state_panic = &state_data.panic;

        let mut last_time = 0;

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

                let timed = self.compute_latest_timed(input_subproperty_index, state_id)?;
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

        let result = func.call(input_values.clone());

        let input_values = input_values.into_iter().map(|e| MetaWrap(e)).collect();

        let AbstractValue::Boolean(result) = result else {
            panic!("Result should be abstract Boolean");
        };

        let valuation = ParamValuation::from_three_valued(result.into_three_valued());

        Ok(TimedCheckValue {
            time: last_time,
            value: CheckValue {
                valuation,
                choice: CheckChoice::Func(input_values),
            },
        })
    }

    /*pub(super) fn compute_negation(
        &self,
        inner: usize,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        let timed = self.compute_latest_timed(inner, state_id)?;
        let timed = Self::apply_negation(timed);

        Ok(timed)
    }

    pub fn apply_negation(mut timed: TimedCheckValue) -> TimedCheckValue {
        timed.value = match timed.value {
            CheckValue::False => CheckValue::True,
            CheckValue::True => CheckValue::False,
            CheckValue::Dependent => {
                // no change
                CheckValue::Dependent
            }
            CheckValue::Unknown(choices) => {
                // no change
                CheckValue::Unknown(choices)
            }
        };

        timed
    }

    pub(super) fn compute_binary_op(
        &self,
        op: &BiLogicOperator,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        let timed_a = self.compute_latest_timed(op.a, state_id)?;
        let timed_b = self.compute_latest_timed(op.b, state_id)?;

        let choice = Self::choose_binary_op(op, &timed_a, &timed_b);

        let mut timed = match choice {
            BiChoice::Left => timed_a,
            BiChoice::Right => timed_b,
        };

        // add the choice made
        if let CheckValue::Unknown(choices) = &mut timed.value {
            choices.push(CheckChoice::BiLogic(choice));
        };

        Ok(timed)
    }

    pub fn choose_binary_op(
        op: &machine_check_common::property::BiLogicOperator,
        timed_a: &TimedCheckValue,
        timed_b: &TimedCheckValue,
    ) -> BiChoice {
        let a_valuation = timed_a.value.valuation();
        let b_valuation = timed_b.value.valuation();

        // use timing to freeze decision
        if a_valuation == b_valuation {
            if timed_a.time <= timed_b.time {
                // choose A
                return BiChoice::Left;
            } else {
                // choose B
                return BiChoice::Right;
            }
        }

        let ordering = if op.is_and {
            a_valuation.upward_bitand_ordering(&b_valuation)
        } else {
            a_valuation.upward_bitor_ordering(&b_valuation)
        };

        match ordering {
            Ordering::Less => BiChoice::Right,
            Ordering::Equal => unreachable!(),
            Ordering::Greater => BiChoice::Left,
        }
    }*/
}
