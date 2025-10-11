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
    pub fn into_property_iir(self) -> IProperty {
        let mut next_var_id: usize = 0;

        let mut data = FromWirData { next_var_id };

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
}
