use std::collections::BTreeMap;

use crate::iir::variable::IVarId;

#[derive(Clone, Debug)]
pub enum IAbstractValue {
    Bitvector(mck::abstr::RBitvector),
    Bool(mck::abstr::Boolean),
    PanicResult(mck::abstr::PanicResult<mck::abstr::RBitvector>),
}

impl IAbstractValue {
    pub fn expect_bitvector(&self) -> mck::abstr::RBitvector {
        let IAbstractValue::Bitvector(bitvec) = self else {
            panic!("Value is not a bitvector");
        };
        *bitvec
    }
}

#[derive(Debug)]
pub struct Interpretation {
    abstract_values: BTreeMap<IVarId, IAbstractValue>,
}

impl Interpretation {
    pub fn new() -> Self {
        Self {
            abstract_values: BTreeMap::new(),
        }
    }

    pub fn abstract_value(&self, var_id: IVarId) -> &IAbstractValue {
        if let Some(value) = self.abstract_values.get(&var_id) {
            value
        } else {
            panic!("Variable {:?} should have interpretation value", var_id)
        }
    }

    pub fn insert_value(&mut self, var_id: IVarId, value: IAbstractValue) {
        if self.abstract_values.insert(var_id, value).is_some() {
            panic!("Variable should never have interpretation value inserted twice");
        }
    }
}

impl Default for Interpretation {
    fn default() -> Self {
        Self::new()
    }
}
