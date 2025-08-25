use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

use machine_check_common::property::NextOperator;
use machine_check_common::{ExecError, NodeId, ParamValuation, StateId};

use crate::model_check::property_checker::labelling_cacher::LabellingCacher;
use crate::model_check::property_checker::{CheckValue, TimedCheckValue};
use crate::FullMachine;

impl<M: FullMachine> LabellingCacher<'_, M> {
    pub fn compute_next_labelling(
        &self,
        op: &NextOperator,
        node_id: NodeId,
    ) -> Result<TimedCheckValue, ExecError> {
        // cache inner labellings of successors
        for successor_id in self.space.direct_successor_iter(node_id) {
            self.compute_latest_timed(op.inner, successor_id)?;
        }

        self.apply_next(op, node_id, &mut BTreeMap::new())
    }

    pub fn apply_next(
        &self,
        op: &NextOperator,
        node_id: NodeId,
        computed_successors: &mut BTreeMap<StateId, TimedCheckValue>,
    ) -> Result<TimedCheckValue, ExecError> {
        // for speed, try to find the appropriate successor without sorting first
        // this can be no successor at all, if the ground value remains,
        // or a single successor with the appropriate valuation
        let Some(tail_partition) = self.space.direct_successor_param_partition(node_id) else {
            // no successors, return the ground value
            return Ok(TimedCheckValue::new(
                0,
                CheckValue::eigen(ParamValuation::from_bool(op.is_universal)),
            ));
        };

        let mut can_be_false_at_time: Option<u64> = None;
        let mut can_be_true_at_time: Option<u64> = None;
        let mut can_be_unknown_at_time: Option<u64> = None;

        let mut best_unknown_successor = None;

        fn update_flag(flag: &mut Option<u64>, time: u64) {
            *flag = Some(flag.map(|flag_time| flag_time.min(time)).unwrap_or(time));
        }

        for parametric_set in tail_partition.all_sets() {
            let (set_valuation, set_valuation_time, best_set_successor) =
                self.compute_set_valuation(op, parametric_set, computed_successors)?;

            match set_valuation {
                ParamValuation::False => {
                    update_flag(&mut can_be_false_at_time, set_valuation_time);
                }
                ParamValuation::True => {
                    update_flag(&mut can_be_true_at_time, set_valuation_time);
                }
                ParamValuation::Dependent => {
                    update_flag(&mut can_be_false_at_time, set_valuation_time);
                    update_flag(&mut can_be_true_at_time, set_valuation_time);
                }
                ParamValuation::Unknown => {
                    if can_be_unknown_at_time.is_some() {
                        best_unknown_successor = None;
                    } else if let Some(best_set_successor) = best_set_successor {
                        best_unknown_successor = Some(best_set_successor);
                    }
                    update_flag(&mut can_be_unknown_at_time, set_valuation_time);
                }
            }
        }

        if let (Some(can_be_false_at_time), Some(can_be_true_at_time)) =
            (can_be_false_at_time, can_be_true_at_time)
        {
            // we know that the valuation definitely depends on the parameter
            let time = can_be_false_at_time.min(can_be_true_at_time);
            return Ok(TimedCheckValue::new(
                time,
                CheckValue::eigen(ParamValuation::Dependent),
            ));
        }

        if can_be_unknown_at_time.is_none() {
            if let Some(time) = can_be_false_at_time {
                return Ok(TimedCheckValue::new(
                    time,
                    CheckValue::eigen(ParamValuation::False),
                ));
            }
            if let Some(time) = can_be_true_at_time {
                return Ok(TimedCheckValue::new(
                    time,
                    CheckValue::eigen(ParamValuation::True),
                ));
            }
        }

        // the valuation is definitely unknown
        // try to return the best unknown successor first

        assert!(can_be_unknown_at_time.is_some());

        if let Some(successor_id) = best_unknown_successor {
            // single allowed successor
            // add the successor id to next states to obtain our value
            let mut timed = computed_successors
                .get(&successor_id)
                .expect("Successor value should be computed")
                .clone();

            assert!(timed.value.valuation.is_unknown());

            timed.value.next_states.push(successor_id);
            return Ok(timed);
        };

        // no best unknown successor
        // we have to sort the successors that have the given valuation

        let mut successor_sorter = Vec::new();

        for successor_id in self.space.direct_successor_iter(node_id) {
            let successor_timed = computed_successors
                .get(&successor_id)
                .expect("Successor value should be computed");
            if successor_timed.value.valuation.is_unknown() {
                successor_sorter.push((
                    (successor_timed.time, successor_id),
                    successor_timed.value.clone(),
                ));
            }
        }

        successor_sorter.sort_by(|(a_key, _), (b_key, _)| a_key.cmp(b_key));

        // the first successor is the wanted one

        let ((successor_time, successor_id), successor_value) = successor_sorter
            .first()
            .expect("There should be a first successor");

        // add the successor id to next states to obtain our value
        let mut timed = TimedCheckValue::new(*successor_time, successor_value.clone());

        assert!(timed.value.valuation.is_unknown());

        timed.value.next_states.push(*successor_id);

        Ok(timed)
    }

    fn compute_set_valuation(
        &self,
        op: &NextOperator,
        parametric_set: partitions::partition_vec::Set<'_, StateId>,
        computed_successors: &mut BTreeMap<StateId, TimedCheckValue>,
    ) -> Result<(ParamValuation, u64, Option<StateId>), ExecError> {
        let ground_value = CheckValue::eigen(ParamValuation::from_bool(op.is_universal));

        let mut current_valuation = ground_value.valuation;

        let mut best_successor = None;

        let mut valuation_time = 0;

        for successor_id in parametric_set.map(|(_, successor_id)| *successor_id) {
            if let Entry::Vacant(e) = computed_successors.entry(successor_id) {
                e.insert(self.compute_latest_timed(op.inner, successor_id)?);
            }

            let successor_timed = computed_successors
                .get(&successor_id)
                .expect("Successor value should be computed");
            let successor_valuation = successor_timed.value.valuation;

            let is_better = if op.is_universal {
                successor_valuation
                    .upward_bitand_ordering(&current_valuation)
                    .is_gt()
            } else {
                successor_valuation
                    .upward_bitor_ordering(&current_valuation)
                    .is_gt()
            };

            if is_better {
                current_valuation = successor_valuation;
                best_successor = Some(successor_id);
                valuation_time = successor_timed.time;
            } else if successor_valuation == current_valuation {
                best_successor = None;
                valuation_time = valuation_time.min(successor_timed.time);
            }
        }

        Ok((current_valuation, valuation_time, best_successor))
    }
}
