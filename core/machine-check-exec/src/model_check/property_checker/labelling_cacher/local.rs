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
        let mut timed = self.compute_latest_timed(inner, state_id)?;
        timed.value.valuation = !timed.value.valuation;
        Ok(timed)
    }

    pub(super) fn compute_binary_op(
        &self,
        op: &BiLogicOperator,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        let timed_a = self.compute_latest_timed(op.a, state_id)?;
        let timed_b = self.compute_latest_timed(op.b, state_id)?;

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
    }
}
