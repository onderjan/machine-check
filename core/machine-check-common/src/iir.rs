use std::collections::BTreeMap;

use proc_macro2::Ident;

use {
    func::IFn,
    interpretation::{IAbstractValue, IRefinementValue, Interpretation},
};

pub mod expr;
pub mod func;
pub mod interpretation;
pub mod path;
pub mod stmt;
pub mod ty;
pub mod variable;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ISubpropertyTypeNext {
    pub universal: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ISubpropertyTypeFixedPoint {
    pub universal: bool,
    pub variable: Ident,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ISubpropertyType {
    Root,
    Next(ISubpropertyTypeNext),
    FixedPoint(ISubpropertyTypeFixedPoint),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ISubpropertyInfo {
    pub ty: ISubpropertyType,
    pub children: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ISubproperty {
    pub func: IFn,
    pub info: ISubpropertyInfo,
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

    pub fn inherent() -> IProperty {
        todo!()
    }
}
