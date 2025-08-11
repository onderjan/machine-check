use std::cmp::Ordering;
use std::collections::BTreeMap;

use machine_check_common::property::BiLogicOperator;
use machine_check_common::{ExecError, StateId};

use crate::model_check::property_checker::labelling_getter::LabellingGetter;
use crate::model_check::property_checker::TimedCheckValue;
use crate::FullMachine;

pub enum BiChoice {
    Left,
    Right,
}

impl<M: FullMachine> LabellingGetter<'_, M> {
    pub(super) fn get_negation(
        &self,
        inner: usize,
        states: impl Iterator<Item = StateId> + Clone,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        Ok(Self::negate(self.get_labelling(inner, states)?))
    }

    pub fn negate(
        mut map: BTreeMap<StateId, TimedCheckValue>,
    ) -> BTreeMap<StateId, TimedCheckValue> {
        // negate everything
        for timed in map.values_mut() {
            timed.value.valuation = !timed.value.valuation;
        }
        map
    }

    pub(super) fn get_binary_op(
        &self,
        op: &BiLogicOperator,
        states: impl Iterator<Item = StateId> + Clone,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        let mut result_a = self.get_labelling(op.a, states.clone())?;
        let mut result_b = self.get_labelling(op.b, states.clone())?;
        let mut result = BTreeMap::new();
        for state_id in states {
            let a_timed = result_a
                .remove(&state_id)
                .expect("Binary operation should get all states from left operand");
            let b_timed = result_b
                .remove(&state_id)
                .expect("Binary operation should get all states from right operand");

            result.insert(state_id, Self::apply_binary_op(op, a_timed, b_timed));
        }
        Ok(result)
    }

    pub fn choose_binary_op(
        op: &machine_check_common::property::BiLogicOperator,
        a_timed: TimedCheckValue,
        b_timed: TimedCheckValue,
    ) -> BiChoice {
        let a_valuation = a_timed.value.valuation;
        let b_valuation = b_timed.value.valuation;

        // use timing to freeze decision
        if a_valuation == b_valuation {
            if a_timed.time <= b_timed.time {
                // choose A
                return BiChoice::Left;
            } else {
                // choose B
                return BiChoice::Right;
            }
        }

        if op.is_and {
            // we prefer the lesser value
            match a_valuation.cmp(&b_valuation) {
                Ordering::Less => BiChoice::Left,
                Ordering::Equal => unreachable!(),
                Ordering::Greater => BiChoice::Right,
            }
        } else {
            // we prefer the greater value
            match a_valuation.cmp(&b_valuation) {
                Ordering::Less => BiChoice::Right,
                Ordering::Equal => unreachable!(),
                Ordering::Greater => BiChoice::Left,
            }
        }
    }

    pub fn apply_binary_op(
        op: &machine_check_common::property::BiLogicOperator,
        a_timed: TimedCheckValue,
        b_timed: TimedCheckValue,
    ) -> TimedCheckValue {
        let a_valuation = a_timed.value.valuation;
        let b_valuation = b_timed.value.valuation;

        // use timing to freeze decision
        if a_valuation == b_valuation {
            if a_timed.time <= b_timed.time {
                return a_timed;
            } else {
                return b_timed;
            }
        }

        if op.is_and {
            // we prefer the lesser value
            match a_valuation.cmp(&b_valuation) {
                Ordering::Less => a_timed,
                Ordering::Equal => unreachable!(),
                Ordering::Greater => b_timed,
            }
        } else {
            // we prefer the greater value
            match a_valuation.cmp(&b_valuation) {
                Ordering::Less => b_timed,
                Ordering::Equal => unreachable!(),
                Ordering::Greater => a_timed,
            }
        }
    }
}
