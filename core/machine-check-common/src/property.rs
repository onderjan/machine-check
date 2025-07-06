//! Computation Tree Logic properties.
use std::fmt::Debug;

use std::sync::Arc;

use crate::ExecError;
use serde::{Deserialize, Serialize};

mod atomic;
mod parser;

pub use atomic::{AtomicProperty, ComparisonType, ValueExpression};

/// A Computation Tree Logic property.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Property {
    arena: Arc<Vec<SubpropertyEntry>>,
}

impl Property {
    pub fn root_subproperty(&self) -> Subproperty {
        Subproperty {
            property: self.clone(),
            index: 0,
        }
    }

    pub fn subproperty_entry(&self, index: usize) -> &SubpropertyEntry {
        self.arena
            .get(index)
            .expect("Subproperty should be within property arena size")
    }

    pub fn inherent() -> Self {
        parser::inherent()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Subproperty {
    property: Property,
    index: usize,
}

impl Subproperty {
    pub fn property(&self) -> &Property {
        &self.property
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn display_str(&self) -> Option<&str> {
        self.property
            .get_by_index(self.index)
            .display_string
            .as_deref()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct SubpropertyEntry {
    pub ty: PropertyType,
    display_string: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum PropertyType {
    Const(bool),
    Atomic(AtomicProperty),
    Negation(usize),
    BiLogicOperator(BiLogicOperator),
    NextOperator(NextOperator),
    LeastFixedPoint(usize),
    GreatestFixedPoint(usize),
    FixedPointVariable(usize),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct BiLogicOperator {
    pub is_and: bool,
    pub a: usize,
    pub b: usize,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct NextOperator {
    pub is_universal: bool,
    pub inner: usize,
}

impl Property {
    pub fn parse(prop_str: &str) -> Result<Property, ExecError> {
        parser::parse(prop_str)
    }

    fn get_by_index(&self, index: usize) -> &SubpropertyEntry {
        self.arena
            .get(index)
            .expect("Subproperty index should be within property arena")
    }
}

impl Subproperty {
    fn new(property: Property, index: usize) -> Self {
        Self { property, index }
    }

    pub fn children(&self) -> Vec<Subproperty> {
        let ty = &self.property.get_by_index(self.index).ty;

        let indices: Vec<usize> = match ty {
            PropertyType::Const(_) => Vec::new(),
            PropertyType::Atomic(_) => Vec::new(),
            PropertyType::Negation(inner) => vec![*inner],
            PropertyType::BiLogicOperator(op) => vec![op.a, op.b],
            PropertyType::NextOperator(op) => vec![op.inner],
            PropertyType::LeastFixedPoint(inner) => {
                vec![*inner]
            }
            PropertyType::GreatestFixedPoint(inner) => {
                vec![*inner]
            }
            PropertyType::FixedPointVariable(_) => Vec::new(),
        };

        indices
            .into_iter()
            .map(|index| Subproperty::new(self.property.clone(), index))
            .collect()
    }
}
