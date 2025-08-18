use std::collections::BTreeMap;

use crate::{iir::variable::IVarId, wir::WMckBinaryOp};

#[derive(Clone, Debug)]
pub struct FBitvector {
    pub width: u32,
    pub inner: mck::abstr::Bitvector<64>,
}

#[derive(Clone, Debug)]
pub enum IValue {
    Bitvector(FBitvector),
    Bool(mck::abstr::Boolean),
}

impl IValue {
    pub fn expect_bitvector(&self) -> &FBitvector {
        let IValue::Bitvector(bitvec) = self else {
            panic!("Value is not a bitvector");
        };
        bitvec
    }
}

#[derive(Debug)]
pub struct Interpretation {
    values: BTreeMap<IVarId, IValue>,
}

impl Interpretation {
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    pub fn value(&self, var_id: IVarId) -> &IValue {
        if let Some(value) = self.values.get(&var_id) {
            value
        } else {
            panic!("Variable {:?} should have interpretation value", var_id)
        }
    }

    pub fn insert_value(&mut self, var_id: IVarId, value: IValue) {
        if self.values.insert(var_id, value).is_some() {
            panic!("Variable should never have interpretation value inserted twice");
        }
    }
}
