use std::collections::BTreeMap;
use std::fmt::Debug;

use crate::misc::Join;

pub trait InterpretationIndex: Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Debug {}

impl<T: Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Debug> InterpretationIndex for T {}

#[derive(Debug)]
pub struct Interpretation<I: InterpretationIndex, V: Join + Debug> {
    values: BTreeMap<I, V>,
}

impl<I: InterpretationIndex, V: Join + Debug> Interpretation<I, V> {
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    pub fn value(&self, var_id: I) -> &V {
        if let Some(value) = self.value_opt(var_id) {
            value
        } else {
            panic!("Variable {:?} should have interpretation value", var_id)
        }
    }

    pub fn value_opt(&self, var_id: I) -> Option<&V> {
        self.values.get(&var_id)
    }

    pub fn insert_value(&mut self, var_id: I, value: V) {
        //eprintln!("Inserting {:?} -> {:?} to {:?}", var_id, value, self);
        if self.values.insert(var_id, value).is_some() {
            panic!("Interpretation value should not be inserted twice");
        }
    }

    pub fn join_value(&mut self, var_id: I, value: V) {
        let value = if let Some(prev_value) = self.values.remove(&var_id) {
            prev_value.join(&value)
        } else {
            value
        };
        self.values.insert(var_id, value);
    }
}

impl<I: InterpretationIndex, V: Join + Debug> Default for Interpretation<I, V> {
    fn default() -> Self {
        Self::new()
    }
}
