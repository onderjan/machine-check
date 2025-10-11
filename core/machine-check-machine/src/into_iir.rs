use std::collections::{BTreeMap, BTreeSet};

use machine_check_common::iir::{
    func::IGlobal, path::IIdent, variable::IVarId, ICalculusOperator, IProperty, ISubproperty,
};

use crate::{
    abstr::YAbstr,
    wir::{WElementaryType, WIdent, WProperty, WSubproperty},
};

mod expr;
mod func;
mod path;
mod stmt;
mod ty;

impl WProperty<YAbstr> {
    pub fn into_property_iir(self) -> IProperty {
        let mut data = FromWirData { next_var_id: 0 };

        let mut subproperties = Vec::new();

        for subproperty in self.subproperties {
            let subproperty = match subproperty {
                WSubproperty::Func(item_fn, children) => {
                    let func = item_fn.into_iir(&mut data);
                    ISubproperty::Func(func, children)
                }
                WSubproperty::FixedPoint(fixed_point) => {
                    ISubproperty::FixedPoint(ICalculusOperator {
                        universal: fixed_point.universal,
                        inner: fixed_point.inner,
                    })
                }
                WSubproperty::Next(next) => ISubproperty::Next(ICalculusOperator {
                    universal: next.universal,
                    inner: next.inner,
                }),
            };

            subproperties.push(subproperty);
        }

        IProperty { subproperties }
    }
}

struct FromWirData {
    next_var_id: usize,
}
