use std::collections::{BTreeMap, BTreeSet};

use machine_check_common::iir::{
    IProperty, ISubproperty, ISubpropertyFixedPoint, ISubpropertyFunc, ISubpropertyNext,
};

use crate::wir::{WProperty, WSubproperty, YConverted};

mod expr;
mod func;
mod path;
mod stmt;
mod ty;

impl WProperty<YConverted> {
    pub fn into_property_iir(self) -> IProperty {
        let mut subproperties = Vec::new();

        let mut subproperty_dependencies = BTreeMap::<usize, BTreeSet<usize>>::new();

        let mut subproperty_dependents = BTreeMap::<usize, Vec<usize>>::new();
        for (subproperty_index, subproperty) in self.subproperties.iter().enumerate() {
            subproperty_dependencies.insert(subproperty_index, subproperty.dependencies());
            if let WSubproperty::FixedPoint(_) = subproperty {
                subproperty_dependents.insert(subproperty_index, Vec::new());
            }
        }

        for subproperty_index in 0..self.subproperties.len() {
            for dependency_index in &subproperty_dependencies[&subproperty_index] {
                let dependency = &self.subproperties[*dependency_index];
                // do not add parent to dependents
                if let WSubproperty::FixedPoint(fixed_point) = dependency {
                    if Some(subproperty_index) != fixed_point.parent {
                        subproperty_dependents
                            .get_mut(dependency_index)
                            .expect("Fixed point should have dependents available")
                            .push(subproperty_index);
                    }
                }
            }
        }

        for (subproperty_index, subproperty) in self.subproperties.into_iter().enumerate() {
            let subproperty = match subproperty {
                WSubproperty::Func(subproperty_func) => ISubproperty::Func(ISubpropertyFunc {
                    parent: subproperty_func.parent,
                    func: subproperty_func.func.into_iir(),
                    children: subproperty_func.children,
                    dependencies: subproperty_dependencies
                        .remove(&subproperty_index)
                        .expect("Subproperty should have dependencies available"),
                }),
                WSubproperty::FixedPoint(fixed_point) => {
                    let dependents = subproperty_dependents
                        .remove(&subproperty_index)
                        .expect("Fixed point should have dependents available");

                    ISubproperty::FixedPoint(ISubpropertyFixedPoint {
                        parent: fixed_point.parent,
                        universal: fixed_point.universal,
                        inner: fixed_point.inner,
                        dependents,
                    })
                }
                WSubproperty::Next(next) => ISubproperty::Next(ISubpropertyNext {
                    parent: next.parent,
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
