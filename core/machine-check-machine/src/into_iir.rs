use std::collections::{BTreeMap, BTreeSet};

use machine_check_common::{
    iir::{
        path::IIdent,
        ty::{IGeneralType, IType},
        variable::{IVarId, IVarInfo},
        IGlobal, IProperty,
    },
    ir_common::IrReference,
};

use crate::{
    abstr::YAbstr,
    wir::{WDescription, WElementaryType, WIdent},
};

mod expr;
mod func;
mod path;
mod stmt;
mod ty;

impl WDescription<YAbstr> {
    pub fn into_iir(self, global_ident_types: BTreeMap<WIdent, WElementaryType>) -> IProperty {
        let mut next_var_id: usize = 0;
        let mut processed_globals = BTreeMap::new();
        let mut global_vars = BTreeMap::new();
        for (ident, ty) in global_ident_types {
            let var_id = IVarId(next_var_id);
            next_var_id += 1;
            processed_globals.insert(
                var_id,
                IGlobal {
                    ident: ident.clone().into_iir(),
                    ty: ty.clone().into_iir(),
                },
            );
            let info = IVarInfo {
                ident: ident.clone().into_iir(),
                ty: IGeneralType::Normal(IType {
                    reference: IrReference::None,
                    inner: ty.into_iir(),
                }),
            };

            global_vars.insert(ident.into_iir(), (var_id, info));
        }

        let mut data = FromWirData {
            next_var_id,
            global_vars,
            used_globals: BTreeSet::new(),
        };

        let mut fns = BTreeMap::new();

        for item_impl in self.impls {
            for func in item_impl.impl_item_fns {
                let func = func.into_iir(&mut data);
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
    global_vars: BTreeMap<IIdent, (IVarId, IVarInfo)>,
    used_globals: BTreeSet<IVarId>,
}
