use func::IFn;
use proc_macro2::Span;

use crate::iir::{func::ISignature, path::IIdent};

pub mod expr;
pub mod func;
pub mod interpretation;
pub mod path;
pub mod stmt;
pub mod ty;
pub mod variable;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ICalculusOperator {
    pub universal: bool,
    pub inner: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ISubproperty {
    Func(IFn, Vec<usize>),
    Next(ICalculusOperator),
    FixedPoint(ICalculusOperator),
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
}
