use std::collections::{BTreeMap, BTreeSet};

use machine_check_common::iir::{
    func::IGlobal, path::IIdent, variable::IVarId, IProperty, ISubproperty,
};

use crate::{
    abstr::YAbstr,
    wir::{WElementaryType, WIdent, WProperty},
};

mod expr;
mod func;
mod path;
mod stmt;
mod ty;

impl WProperty<YAbstr> {
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

        for subproperty in self.subproperties {
            let func = subproperty.func.into_iir(&mut data);

            subproperties.push(ISubproperty {
                func,
                info: subproperty.info,
            });
        }

        IProperty { subproperties }
    }
}

struct FromWirData {
    next_var_id: usize,
    global_var_ids: BTreeMap<IIdent, IVarId>,
    global_var_infos: BTreeMap<IVarId, IGlobal>,
    used_globals: BTreeSet<IVarId>,
}
