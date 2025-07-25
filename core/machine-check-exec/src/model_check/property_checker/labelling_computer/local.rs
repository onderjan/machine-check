use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use machine_check_common::property::BiLogicOperator;
use machine_check_common::{ExecError, StateId};

use crate::model_check::property_checker::labelling_computer::LabellingComputer;
use crate::FullMachine;

impl<M: FullMachine> LabellingComputer<'_, M> {
    pub fn compute_negation(
        &mut self,
        subproperty_index: usize,
        inner: usize,
    ) -> Result<BTreeSet<StateId>, ExecError> {
        let dirty = self.compute_labelling(inner)?;

        //let dirty = Self::computation_mut(&mut self.updates, inner);

        // negate everything that was updated
        let mut update = BTreeMap::new();

        let inner_labelling = self.property_checker.get_labelling(inner);

        for state_id in dirty.iter().cloned() {
            let mut value = inner_labelling.get(&state_id).unwrap().clone();
            // negate
            value.valuation = !value.valuation;
            update.insert(state_id, value);
        }

        self.update_subproperty(subproperty_index, update)
    }

    pub fn compute_binary_op(
        &mut self,
        subproperty_index: usize,
        op: &BiLogicOperator,
    ) -> Result<BTreeSet<StateId>, ExecError> {
        let updated_a = self.compute_labelling(op.a)?;
        let updated_b = self.compute_labelling(op.b)?;

        let dirty = updated_a.union(&updated_b).copied();

        let mut updated = BTreeMap::new();

        for state_id in dirty {
            let a_value = self.value(op.a, state_id);
            let b_value = self.value(op.b, state_id);

            let a_valuation = a_value.valuation;
            let b_valuation = b_value.valuation;

            // TODO: freeze decision

            let result_value = if op.is_and {
                // we prefer the lesser value
                match a_valuation.cmp(&b_valuation) {
                    Ordering::Less => a_value,
                    Ordering::Equal => a_value,
                    Ordering::Greater => b_value,
                }
            } else {
                // we prefer the greater value
                match a_valuation.cmp(&b_valuation) {
                    Ordering::Less => b_value,
                    Ordering::Equal => a_value,
                    Ordering::Greater => a_value,
                }
            };

            updated.insert(state_id, result_value.clone());
        }

        self.update_subproperty(subproperty_index, updated)
    }
}
