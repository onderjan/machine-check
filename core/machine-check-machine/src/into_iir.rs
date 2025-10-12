use machine_check_common::iir::{ICalculusOperator, IProperty, ISubproperty};

use crate::{
    abstr::YAbstr,
    wir::{WProperty, WSubproperty},
};

mod expr;
mod func;
mod path;
mod stmt;
mod ty;

impl WProperty<YAbstr> {
    pub fn into_property_iir(self) -> IProperty {
        let mut subproperties = Vec::new();

        for subproperty in self.subproperties {
            let subproperty = match subproperty {
                WSubproperty::Func(item_fn, children) => {
                    let func = item_fn.into_iir();
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

        //log::trace!("Property IIR: {:#?}", subproperties);

        IProperty { subproperties }
    }
}
