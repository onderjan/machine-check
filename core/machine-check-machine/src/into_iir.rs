use std::collections::{BTreeMap, BTreeSet};

use machine_check_common::{
    iir::{
        func::IGlobal,
        path::IIdent,
        ty::{IGeneralType, IType},
        variable::{IVarId, IVarInfo},
        IProperty,
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
    pub fn into_property_iir(
        self,
        global_ident_types: BTreeMap<WIdent, WElementaryType>,
    ) -> IProperty {
        let mut next_var_id: usize = 0;
        let mut used_globals = BTreeMap::new();
        let mut global_var_ids = BTreeMap::new();
        let mut global_var_infos = BTreeMap::new();
        for (ident, ty) in global_ident_types {
            let var_id = IVarId(next_var_id);
            next_var_id += 1;
            used_globals.insert(
                var_id,
                IGlobal {
                    ident: ident.clone().into_iir(),
                    ty: ty.clone().into_iir(),
                },
            );
            let global = IGlobal {
                ident: ident.clone().into_iir(),
                ty: ty.into_iir(),
            };

            global_var_ids.insert(ident.into_iir(), var_id);
            global_var_infos.insert(var_id, global);
        }

        let mut data = FromWirData {
            next_var_id,
            global_var_ids,
            global_var_infos,
            used_globals: BTreeSet::new(),
        };

        let mut subproperties = Vec::new();

        for item_impl in self.impls {
            for func in item_impl.impl_item_fns {
                let func = func.into_iir(&mut data);

                let fn_num: usize = func
                    .signature
                    .ident
                    .name()
                    .strip_prefix("fn_")
                    .expect("Property function should be regularly prefixed")
                    .parse()
                    .expect("Property function should consist of a prefix and a number");
                assert_eq!(fn_num, subproperties.len());

                subproperties.push(func);
            }
        }

        // TODO: only retain used globals
        //used_globals.retain(|var_id, _| data.used_globals.contains(var_id));

        IProperty { subproperties }
    }
}

struct FromWirData {
    next_var_id: usize,
    global_var_ids: BTreeMap<IIdent, IVarId>,
    global_var_infos: BTreeMap<IVarId, IGlobal>,
    used_globals: BTreeSet<IVarId>,
}
