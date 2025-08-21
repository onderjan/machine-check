use std::collections::BTreeMap;

use proc_macro2::Span;

use crate::iir::{path::IIdent, ty::IElementaryType};

use {
    func::IFn,
    interpretation::{IAbstractValue, IRefinementValue, Interpretation},
    variable::IVarId,
};

pub mod expr;
pub mod func;
pub mod interpretation;
pub mod path;
pub mod stmt;
pub mod ty;
pub mod variable;

#[derive(Clone, Debug)]
pub struct IGlobal {
    pub ident: IIdent,
    pub ty: IElementaryType,
}

#[derive(Clone, Debug)]
pub struct IProperty {
    pub globals: BTreeMap<IVarId, IGlobal>,
    pub fns: BTreeMap<IIdent, IFn>,
}

impl IProperty {
    pub fn forward_interpret_fn(
        &self,
        fn_name: String,
        global_abstract_values: &BTreeMap<String, IAbstractValue>,
    ) -> IAbstractValue {
        let fn_ident = IIdent::new(fn_name.clone(), Span::call_site());
        let Some(func) = self.fns.get(&fn_ident) else {
            panic!("Unable to find function '{}' to forward-interpret", fn_name);
        };

        let mut inter = Interpretation::new();

        println!("Property globals: {:?}", self.globals);
        for (var_id, global) in &self.globals {
            if let Some(global_value) = global_abstract_values.get(global.ident.name()) {
                inter.insert_abstract_value(*var_id, global_value.clone());
            }
        }

        println!("Forward-interpreting function {:#?}", func);

        func.forward_interpret(&mut inter);

        println!("Forward function interpretation: {:#?}", inter);

        let normal_result = inter.abstract_value(func.signature.output.normal).clone();
        let panic_result = inter
            .abstract_value(func.signature.output.panic)
            .expect_bitvector();
        assert!(panic_result.concrete_value().is_some_and(|v| v.is_zero()));
        normal_result
    }

    pub fn backward_interpret_fn(
        &self,
        fn_name: String,
        global_abstract_values: &BTreeMap<String, IAbstractValue>,
        result_refinement_value: IRefinementValue,
        panic_refinement_value: mck::refin::RBitvector,
    ) -> BTreeMap<String, IRefinementValue> {
        let fn_ident = IIdent::new(fn_name.clone(), Span::call_site());
        let Some(func) = self.fns.get(&fn_ident) else {
            panic!(
                "Unable to find function '{}' to backward-interpret",
                fn_name
            );
        };

        let mut inter = Interpretation::new();

        println!("Property globals: {:?}", self.globals);
        for (var_id, global) in &self.globals {
            if let Some(global_value) = global_abstract_values.get(global.ident.name()) {
                inter.insert_abstract_value(*var_id, global_value.clone());
            }
        }

        inter.insert_refinement_value(func.signature.output.normal, result_refinement_value);
        inter.insert_refinement_value(
            func.signature.output.panic,
            IRefinementValue::Bitvector(panic_refinement_value),
        );

        println!(
            "Forward-interpreting function {:#?} before backward interpretation",
            func
        );

        func.forward_interpret(&mut inter);

        println!("Forward function interpretation: {:#?}", inter);

        println!("Backward-interpreting function {:#?}", func);

        func.backward_interpret(&mut inter);

        println!("Backward function interpretation: {:#?}", inter);

        BTreeMap::new()
    }
}
