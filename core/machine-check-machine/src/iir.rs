use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Span;

use crate::{
    abstr::YAbstr,
    iir::{
        func::IFn,
        interpretation::{IValue, Interpretation},
        variable::IVarId,
    },
    wir::{WBasicType, WDescription, WElementaryType, WIdent, WProperty, WTypeArray},
};

use mck::abstr::Field;

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
        assert_eq!(
            panic_result.inner.concrete_value(),
            Some(mck::concr::Bitvector::new(0))
        );
        normal_result
    }

    pub fn from_wir(
        description: WDescription<YAbstr>,
        globals: BTreeMap<String, mck::abstr::Field>,
    ) -> IProperty {
        let mut next_var_id: usize = 0;
        let mut processed_globals = BTreeMap::new();
        let mut global_var_ids = BTreeMap::new();
        for (name, field) in globals {
            let ident = WIdent::new(name, Span::call_site());
            let ty = match &field {
                Field::Bitvector(field) => WElementaryType::Bitvector(field.bit_width),
                Field::Array(field) => WElementaryType::Array(WTypeArray {
                    index_width: field.bit_length,
                    element_width: field.bit_width,
                }),
            };
            let var_id = IVarId(next_var_id);
            next_var_id += 1;
            processed_globals.insert(
                var_id,
                IGlobal {
                    ident: ident.clone(),
                    ty,
                },
            );
            global_var_ids.insert(ident, var_id);
        }

        let mut data = FromWirData {
            next_var_id,
            global_var_ids,
            used_globals: BTreeSet::new(),
        };

        let mut fns = BTreeMap::new();

        for item_impl in description.impls {
            for func in item_impl.impl_item_fns {
                let func = IFn::from_wir(&mut data, func);
                fns.insert(func.signature.ident.clone(), func);
            }
        }

        processed_globals.retain(|var_id, _| data.used_globals.contains(var_id));

        IProperty {
            globals: processed_globals,
            fns,
        }
    }
}

struct FromWirData {
    next_var_id: usize,
    global_var_ids: BTreeMap<WIdent, IVarId>,
    used_globals: BTreeSet<IVarId>,
}
