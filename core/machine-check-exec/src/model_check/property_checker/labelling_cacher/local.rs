use std::cmp::Ordering;

use machine_check_common::property::BiLogicOperator;
use machine_check_common::{ExecError, StateId};

use crate::model_check::property_checker::labelling_cacher::LabellingCacher;
use crate::model_check::property_checker::TimedCheckValue;
use crate::FullMachine;

pub enum BiChoice {
    Left,
    Right,
}

impl<M: FullMachine> LabellingCacher<'_, M> {
    pub(super) fn compute_negation(
        &self,
        inner: usize,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        self.cache_if_uncached(inner, state_id)?;
        let mut timed = self.property_checker.get_cached(inner, state_id);

        timed.value.valuation = !timed.value.valuation;
        Ok(timed)
    }

    pub(super) fn compute_binary_op(
        &self,
        op: &BiLogicOperator,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        self.cache_if_uncached(op.a, state_id)?;
        self.cache_if_uncached(op.b, state_id)?;

        let timed_a = self.property_checker.get_cached(op.a, state_id);
        let timed_b = self.property_checker.get_cached(op.b, state_id);

        Ok(match Self::choose_binary_op(op, &timed_a, &timed_b) {
            BiChoice::Left => timed_a,
            BiChoice::Right => timed_b,
        })
    }

    pub fn choose_binary_op(
        op: &machine_check_common::property::BiLogicOperator,
        timed_a: &TimedCheckValue,
        timed_b: &TimedCheckValue,
    ) -> BiChoice {
        let a_valuation = timed_a.value.valuation;
        let b_valuation = timed_b.value.valuation;

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
}
