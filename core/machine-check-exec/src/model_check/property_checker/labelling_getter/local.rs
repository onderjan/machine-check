use std::cmp::Ordering;

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
    pub(super) fn evaluate_negation(
        &self,
        inner: usize,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        self.cache_labelling(inner, state_id)?;
        let mut timed = self.property_checker.get_cached(inner, state_id);

        timed.value.valuation = !timed.value.valuation;
        Ok(timed)
    }

    pub(super) fn evaluate_binary_op(
        &self,
        op: &BiLogicOperator,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        self.cache_labelling(op.a, state_id)?;
        self.cache_labelling(op.b, state_id)?;

        let timed_a = self.property_checker.get_cached(op.a, state_id);
        let timed_b = self.property_checker.get_cached(op.b, state_id);

        Ok(Self::apply_binary_op(op, timed_a, timed_b))
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
        timed_a: TimedCheckValue,
        timed_b: TimedCheckValue,
    ) -> TimedCheckValue {
        let a_valuation = timed_a.value.valuation;
        let b_valuation = timed_b.value.valuation;

        // use timing to freeze decision
        if a_valuation == b_valuation {
            if timed_a.time <= timed_b.time {
                return timed_a;
            } else {
                return timed_b;
            }
        }

        if op.is_and {
            // we prefer the lesser value
            match a_valuation.cmp(&b_valuation) {
                Ordering::Less => timed_a,
                Ordering::Equal => unreachable!(),
                Ordering::Greater => timed_b,
            }
        } else {
            // we prefer the greater value
            match a_valuation.cmp(&b_valuation) {
                Ordering::Less => timed_b,
                Ordering::Equal => unreachable!(),
                Ordering::Greater => timed_a,
            }
        }
    }
}
