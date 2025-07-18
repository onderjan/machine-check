use std::cmp::Ordering;
use std::collections::BTreeMap;

use machine_check_common::property::BiLogicOperator;
use machine_check_common::ExecError;

use crate::model_check::property_checker::labelling_computer::LabellingComputer;
use crate::FullMachine;

impl<M: FullMachine> LabellingComputer<'_, M> {
    pub fn compute_negation(
        &mut self,
        subproperty_index: usize,
        inner: usize,
    ) -> Result<(), ExecError> {
        self.compute_labelling(inner)?;

        let inner_computation = Self::computation_mut(&mut self.computations, inner);

        // negate everything that was updated
        let mut update = BTreeMap::new();
        for state_id in &inner_computation.updated {
            let mut value = inner_computation.values.get(state_id).unwrap().clone();
            // negate
            value.valuation = !value.valuation;
            update.insert(*state_id, value);
        }

        inner_computation.updated.clear();

        self.update_subproperty(subproperty_index, update);
        Ok(())
    }

    pub fn compute_binary_op(
        &mut self,
        subproperty_index: usize,
        op: &BiLogicOperator,
    ) -> Result<(), ExecError> {
        self.compute_labelling(op.a)?;
        self.compute_labelling(op.b)?;

        let a_computation = Self::computation(&self.computations, op.a);
        let b_computation = Self::computation(&self.computations, op.b);

        let mut dirty = a_computation.updated.clone();
        dirty.extend(b_computation.updated.iter());

        let mut update = BTreeMap::new();

        for state_id in dirty {
            let a_value = a_computation
                .values
                .get(&state_id)
                .expect("Binary operation should have left value available");
            let b_value = b_computation
                .values
                .get(&state_id)
                .expect("Binary operation should have right value available");

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

            update.insert(state_id, result_value.clone());
        }

        let a_computation = Self::computation_mut(&mut self.computations, op.a);
        a_computation.updated.clear();
        let b_computation = Self::computation_mut(&mut self.computations, op.b);
        b_computation.updated.clear();

        self.update_subproperty(subproperty_index, update);

        Ok(())
    }
}
