mod abstr;
mod refin;

use std::collections::BTreeMap;
use std::fmt::Debug;

use crate::iir::variable::IVarId;

pub use {abstr::IAbstractValue, refin::IRefinementValue};

pub trait Join {
    fn join(&self, other: &Self) -> Self;
}

#[derive(Debug)]
pub struct Interpretation<V: Join + Debug> {
    values: BTreeMap<IVarId, V>,
}

impl<V: Join + Debug> Interpretation<V> {
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    pub fn value(&self, var_id: IVarId) -> &V {
        if let Some(value) = self.value_opt(var_id) {
            value
        } else {
            panic!("Variable {:?} should have interpretation value", var_id)
        }
    }

    pub fn value_opt(&self, var_id: IVarId) -> Option<&V> {
        self.values.get(&var_id)
    }

    pub(super) fn insert_value(&mut self, var_id: IVarId, value: V) {
        //eprintln!("Inserting {:?} -> {:?} to {:?}", var_id, value, self);
        if self.values.insert(var_id, value).is_some() {
            panic!("Interpretation value should not be inserted twice");
        }
    }

    pub(super) fn join_value(&mut self, var_id: IVarId, value: V) {
        let value = if let Some(prev_value) = self.values.remove(&var_id) {
            prev_value.join(&value)
        } else {
            value
        };
        self.values.insert(var_id, value);
    }
}

impl<V: Join + Debug> Default for Interpretation<V> {
    fn default() -> Self {
        Self::new()
    }
}
