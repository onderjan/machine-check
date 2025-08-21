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

#[derive(Clone, Debug)]
pub enum IRefinementValue {
    Bitvector(mck::refin::RBitvector),
    Bool(mck::refin::Boolean),
    PanicResult(mck::refin::PanicResult<mck::refin::RBitvector>),
}

impl IRefinementValue {
    pub fn expect_bitvector(&self) -> mck::refin::RBitvector {
        let IRefinementValue::Bitvector(result) = self else {
            panic!("Value is not a bitvector");
        };
        *result
    }

    pub fn expect_boolean(&self) -> mck::refin::Boolean {
        let IRefinementValue::Bool(result) = self else {
            panic!("Value is not a Boolean");
        };
        *result
    }
}

#[derive(Debug)]
pub struct Interpretation {
    abstract_values: BTreeMap<IVarId, IAbstractValue>,
    refinement_values: BTreeMap<IVarId, IRefinementValue>,
}

impl Interpretation {
    pub fn new() -> Self {
        Self {
            abstract_values: BTreeMap::new(),
            refinement_values: BTreeMap::new(),
        }
    }

    pub fn abstract_value(&self, var_id: IVarId) -> &IAbstractValue {
        if let Some(value) = self.abstract_values.get(&var_id) {
            value
        } else {
            panic!(
                "Variable {:?} should have abstract interpretation value",
                var_id
            )
        }
    }

    pub(super) fn insert_abstract_value(&mut self, var_id: IVarId, value: IAbstractValue) {
        if self.abstract_values.insert(var_id, value).is_some() {
            panic!("Variable should never have abstract interpretation value inserted twice");
        }
    }

    pub fn refinement_value_opt(&self, var_id: IVarId) -> Option<IRefinementValue> {
        self.refinement_values.get(&var_id).cloned()
    }

    pub(super) fn insert_refinement_value(&mut self, var_id: IVarId, value: IRefinementValue) {
        if self.refinement_values.insert(var_id, value).is_some() {
            panic!("Variable should never have refinement interpretation value inserted twice");
        }
    }

    pub(super) fn compute_abstract_value(&mut self, var_id: IVarId) -> IAbstractValue {
        if let Some(value) = self.abstract_values.get(&var_id) {
            value.clone()
        } else {
            // TODO: do something with partial computation
            panic!("Abstract value of variable {:?} should be computed", var_id)
        }
    }
}

impl Default for Interpretation {
    fn default() -> Self {
        Self::new()
    }
}
