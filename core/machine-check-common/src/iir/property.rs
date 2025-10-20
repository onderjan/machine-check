use std::collections::BTreeSet;

use super::func::IFn;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ISubpropertyFunc {
    pub parent: Option<usize>,
    pub func: IFn,
    pub children: Vec<usize>,
    pub dependencies: BTreeSet<usize>,
    pub display: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ISubpropertyNext {
    pub parent: Option<usize>,
    pub universal: bool,
    pub inner: usize,
    pub display: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ISubpropertyFixedPoint {
    pub parent: Option<usize>,
    pub universal: bool,
    pub inner: usize,
    pub dependents: Vec<usize>,
    pub display: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ISubproperty {
    Func(ISubpropertyFunc),
    Next(ISubpropertyNext),
    FixedPoint(ISubpropertyFixedPoint),
}

impl ISubproperty {
    pub fn parent(&self) -> Option<usize> {
        match self {
            ISubproperty::Func(subproperty) => subproperty.parent,
            ISubproperty::Next(subproperty) => subproperty.parent,
            ISubproperty::FixedPoint(subproperty) => subproperty.parent,
        }
    }

    pub fn children(&self) -> &[usize] {
        match self {
            ISubproperty::Func(subproperty) => &subproperty.children,
            ISubproperty::Next(subproperty) => std::slice::from_ref(&subproperty.inner),
            ISubproperty::FixedPoint(subproperty) => std::slice::from_ref(&subproperty.inner),
        }
    }

    pub fn display_str(&self) -> &str {
        match self {
            ISubproperty::Func(subproperty) => &subproperty.display,
            ISubproperty::Next(subproperty) => &subproperty.display,
            ISubproperty::FixedPoint(subproperty) => &subproperty.display,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IProperty {
    pub subproperties: Vec<ISubproperty>,
}

impl IProperty {
    pub fn num_subproperties(&self) -> usize {
        self.subproperties.len()
    }

    pub fn subproperty_entry(&self, index: usize) -> &ISubproperty {
        &self.subproperties[index]
    }

    pub fn transition_depth(&self) -> usize {
        let (open_number, max_closed_number) = self.subproperty_transition_depth(0);

        open_number.max(max_closed_number)
    }

    fn subproperty_transition_depth(&self, subproperty_index: usize) -> (usize, usize) {
        let subproperty = &self.subproperties[subproperty_index];
        match subproperty {
            ISubproperty::Func(subproperty) => {
                // maximum of children
                let mut open_number = 0;
                let mut max_closed_number = 0;

                for child_index in subproperty.children.iter().cloned() {
                    let (candidate_open_number, candidate_max_closed_number) =
                        self.subproperty_transition_depth(child_index);

                    open_number = open_number.max(candidate_open_number);
                    max_closed_number = max_closed_number.max(candidate_max_closed_number);
                }

                (open_number, max_closed_number)
            }
            ISubproperty::Next(subproperty) => {
                // increment open number
                let (mut open_number, max_closed_number) =
                    self.subproperty_transition_depth(subproperty.inner);
                open_number += 1;
                (open_number, max_closed_number)
            }
            ISubproperty::FixedPoint(subproperty) => {
                // close the opening
                let (open_number, max_closed_number) =
                    self.subproperty_transition_depth(subproperty.inner);
                let max_closed_number = max_closed_number.max(open_number);
                (0, max_closed_number)
            }
        }
    }

    pub fn is_subproperty_closed_form(&self, subproperty_index: usize) -> bool {
        let mut self_descendants = BTreeSet::new();
        let mut dependencies = BTreeSet::new();

        self.compute_self_descendants_and_dependencies(
            subproperty_index,
            &mut self_descendants,
            &mut dependencies,
        );
        self_descendants == dependencies
    }

    fn compute_self_descendants_and_dependencies(
        &self,
        subproperty_index: usize,
        self_descendants: &mut BTreeSet<usize>,
        dependencies: &mut BTreeSet<usize>,
    ) {
        self_descendants.insert(subproperty_index);
        dependencies.insert(subproperty_index);
        let subproperty = &self.subproperties[subproperty_index];
        match subproperty {
            ISubproperty::Func(subproperty) => {
                dependencies.extend(subproperty.dependencies.iter().cloned());
                for child_index in subproperty.children.iter().cloned() {
                    self.compute_self_descendants_and_dependencies(
                        child_index,
                        self_descendants,
                        dependencies,
                    );
                }
            }
            ISubproperty::Next(subproperty) => {
                self.compute_self_descendants_and_dependencies(
                    subproperty.inner,
                    self_descendants,
                    dependencies,
                );
            }
            ISubproperty::FixedPoint(subproperty) => {
                self.compute_self_descendants_and_dependencies(
                    subproperty.inner,
                    self_descendants,
                    dependencies,
                );
            }
        };
    }
}
