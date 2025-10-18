use std::collections::BTreeSet;

use func::IFn;

pub mod expr;
pub mod func;
pub mod interpretation;
pub mod path;
pub mod stmt;
pub mod ty;
pub mod variable;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ISubpropertyFunc {
    pub parent: Option<usize>,
    pub func: IFn,
    pub children: Vec<usize>,
    pub dependencies: BTreeSet<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ISubpropertyNext {
    pub parent: Option<usize>,
    pub universal: bool,
    pub inner: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ISubpropertyFixedPoint {
    pub parent: Option<usize>,
    pub universal: bool,
    pub inner: usize,
    pub dependents: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
        // TODO: compute transition depth
        5
    }
}
