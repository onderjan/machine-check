use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Span;

use crate::{
    abstr::YAbstr,
    iir::{
        func::IFn,
        interpretation::{IAbstractValue, IRefinementValue, Interpretation},
        variable::{IVarId, IVarInfo},
    },
    wir::{WDescription, WElementaryType, WGeneralType, WIdent, WReference, WType},
};

mod expr;
mod func;
pub mod interpretation;
mod stmt;
mod variable;

#[derive(Clone, Debug)]
pub struct IGlobal {
    ident: WIdent,
    ty: WElementaryType,
}

#[derive(Clone, Debug)]
pub struct IProperty {
    pub globals: BTreeMap<IVarId, IGlobal>,
    pub fns: BTreeMap<WIdent, IFn>,
}

impl IProperty {
    pub fn forward_interpret_fn(
        &self,
        fn_name: String,
        global_abstract_values: &BTreeMap<String, IAbstractValue>,
    ) -> IAbstractValue {
        let fn_ident = WIdent::new(fn_name.clone(), Span::call_site());
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
        let fn_ident = WIdent::new(fn_name.clone(), Span::call_site());
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

    pub fn from_wir(
        description: WDescription<YAbstr>,
        global_ident_types: BTreeMap<WIdent, WElementaryType>,
    ) -> IProperty {
        let mut next_var_id: usize = 0;
        let mut processed_globals = BTreeMap::new();
        let mut global_vars = BTreeMap::new();
        for (ident, ty) in global_ident_types {
            let var_id = IVarId(next_var_id);
            next_var_id += 1;
            processed_globals.insert(
                var_id,
                IGlobal {
                    ident: ident.clone(),
                    ty: ty.clone(),
                },
            );
            let info = IVarInfo {
                ident: ident.clone(),
                ty: WGeneralType::Normal(WType {
                    reference: WReference::None,
                    inner: ty,
                }),
            };

            global_vars.insert(ident, (var_id, info));
        }

        let mut data = FromWirData {
            next_var_id,
            global_vars,
            used_globals: BTreeSet::new(),
        };

        let mut fns = BTreeMap::new();

        for item_impl in description.impls {
            for func in item_impl.impl_item_fns {
                let func = IFn::from_wir(&mut data, func);
                fns.insert(func.signature.ident.clone(), func);
            }
        }

        // TODO: only retain used globals
        //processed_globals.retain(|var_id, _| data.used_globals.contains(var_id));

        IProperty {
            globals: processed_globals,
            fns,
        }
    }
}

struct FromWirData {
    next_var_id: usize,
    global_vars: BTreeMap<WIdent, (IVarId, IVarInfo)>,
    used_globals: BTreeSet<IVarId>,
}
