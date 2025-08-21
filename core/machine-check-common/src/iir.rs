use std::collections::BTreeMap;

use {
    func::IFn,
    interpretation::{IAbstractValue, IRefinementValue, Interpretation},
};

pub mod expr;
pub mod func;
pub mod interpretation;
pub mod path;
pub mod stmt;
pub mod ty;
pub mod variable;

#[derive(Clone, Debug)]
pub struct IProperty {
    pub subproperties: Vec<IFn>,
}

impl IProperty {
    pub fn forward_interpret(
        &self,
        global_abstract_values: &BTreeMap<String, IAbstractValue>,
    ) -> IAbstractValue {
        self.forward_interpret_subproperty(global_abstract_values, 0)
    }

    pub fn forward_interpret_subproperty(
        &self,
        global_forward: &BTreeMap<String, IAbstractValue>,
        subproperty_index: usize,
    ) -> IAbstractValue {
        let func = &self.subproperties[subproperty_index];

        let mut inter = Interpretation::new();

        func.forward_interpret(&mut inter, global_forward);

        println!("Forward function interpretation: {:#?}", inter);

        let normal_result = inter.abstract_value(func.signature.output.normal).clone();
        // TODO: raise an error on nonzero panic result
        let panic_result = inter
            .abstract_value(func.signature.output.panic)
            .expect_bitvector();
        assert!(panic_result.concrete_value().is_some_and(|v| v.is_zero()));
        normal_result
    }

    pub fn backward_interpret(
        &self,
        global_forward: &BTreeMap<String, IAbstractValue>,
        result_backward: IRefinementValue,
    ) -> BTreeMap<String, IRefinementValue> {
        self.backward_interpret_subproperty(global_forward, result_backward, 0)
    }

    pub fn backward_interpret_subproperty(
        &self,
        global_forward: &BTreeMap<String, IAbstractValue>,
        result_backward: IRefinementValue,
        subproperty_index: usize,
    ) -> BTreeMap<String, IRefinementValue> {
        let func = &self.subproperties[subproperty_index];

        let mut inter = Interpretation::new();

        inter.insert_refinement_value(func.signature.output.normal, result_backward);
        inter.insert_refinement_value(
            func.signature.output.panic,
            IRefinementValue::Bitvector(mck::refin::RBitvector::new_unmarked(32)),
        );

        func.backward_interpret(&mut inter, global_forward);

        println!("Backward function interpretation: {:#?}", inter);

        BTreeMap::new()
    }
}
