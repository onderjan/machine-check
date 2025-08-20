use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Span;

use crate::{
    abstr::YAbstr,
    iir::{
        func::IFn,
        interpretation::{IValue, Interpretation},
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
    pub fn interpret_fn(
        &self,
        fn_name: String,
        mut global_values: BTreeMap<String, IValue>,
    ) -> IValue {
        let fn_ident = WIdent::new(fn_name.clone(), Span::call_site());
        let Some(func) = self.fns.get(&fn_ident) else {
            panic!("Unable to find function '{}' to interpret", fn_name);
        };

        let mut inter = Interpretation::new();

        println!("Property globals: {:?}", self.globals);
        for (var_id, global) in &self.globals {
            if let Some(global_value) = global_values.remove(global.ident.name()) {
                inter.insert_value(*var_id, global_value);
            }
        }

        println!("Interpreting function {:#?}", func);

        for stmt in &func.block.stmts {
            stmt.interpret(&mut inter);
        }

        println!("Function interpretation: {:#?}", inter);

        let normal_result = inter.value(func.signature.output.normal).clone();
        let panic_result = inter.value(func.signature.output.panic).expect_bitvector();
        assert!(panic_result.concrete_value().is_some_and(|v| v.is_zero()));
        normal_result
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
