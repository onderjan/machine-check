use std::collections::{BTreeMap, BTreeSet};

use indexmap::IndexMap;
use machine_check_common::iir::property::{
    IProperty, ISubproperty, ISubpropertyFixedPoint, ISubpropertyFunc, ISubpropertyNext,
};

use crate::{
    wir::{WContext, WProperty, WSubproperty, YConverted},
    Error,
};

impl WProperty<YConverted> {
    pub fn into_iir(self, ctx: &WContext) -> Result<IProperty, Error> {
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
                WSubproperty::Func(subproperty) => ISubproperty::Func(ISubpropertyFunc {
                    parent: subproperty.parent,
                    func: subproperty.func.into_iir(ctx, &IndexMap::new())?,
                    children: subproperty.children,
                    dependencies: subproperty_dependencies
                        .remove(&subproperty_index)
                        .expect("Subproperty should have dependencies available"),
                    display: subproperty.display,
                }),
                WSubproperty::FixedPoint(subproperty) => {
                    let dependents = subproperty_dependents
                        .remove(&subproperty_index)
                        .expect("Fixed point should have dependents available");

                    ISubproperty::FixedPoint(ISubpropertyFixedPoint {
                        parent: subproperty.parent,
                        universal: subproperty.greatest,
                        inner: subproperty.inner,
                        dependents,
                        display: subproperty.display,
                    })
                }
                WSubproperty::Next(subproperty) => ISubproperty::Next(ISubpropertyNext {
                    parent: subproperty.parent,
                    universal: subproperty.universal,
                    inner: subproperty.inner,
                    display: subproperty.display,
                }),
            };

            subproperties.push(subproperty);
        }

        Ok(IProperty { subproperties })
    }
}
