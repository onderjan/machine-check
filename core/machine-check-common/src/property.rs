//! Computation Tree Logic properties.
use std::{collections::BTreeSet, fmt::Debug};

use std::sync::Arc;

use crate::ExecError;
use serde::{Deserialize, Serialize};

mod atomic;
mod closed_form;
mod parser;
mod transition_depth;

pub use atomic::{AtomicProperty, ComparisonType, ValueExpression};

/// A Computation Tree Logic property.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

    pub fn affected_fixed_points(&self, index: usize) -> BTreeSet<usize> {
        let ty = &self.get_by_index(index).ty;
        match ty {
            PropertyType::Const(_) | PropertyType::Atomic(_) => BTreeSet::new(),
            PropertyType::Negation(inner) => self.affected_fixed_points(*inner),
            PropertyType::BiLogic(bi_logic_operator) => BTreeSet::from_iter(
                self.affected_fixed_points(bi_logic_operator.a)
                    .union(&self.affected_fixed_points(bi_logic_operator.b))
                    .copied(),
            ),
            PropertyType::Next(next_operator) => self.affected_fixed_points(next_operator.inner),
            PropertyType::FixedPoint(fixed_point_operator) => {
                let mut inner_affected = self.affected_fixed_points(fixed_point_operator.inner);
                inner_affected.insert(index);
                inner_affected
            }
            PropertyType::FixedVariable(inner) => BTreeSet::from([*inner]),
        }
    }

    pub fn inherent() -> Self {
        parser::inherent()
    }

    pub fn num_subproperties(&self) -> usize {
        self.arena.len()
    }
}

impl Debug for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Property");
        for (index, entry) in self.arena.iter().enumerate() {
            let index_string = index.to_string();
            builder.field(&index_string, entry);
        }

        builder.finish()
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
    BiLogic(BiLogicOperator),
    Next(NextOperator),
    FixedPoint(FixedPointOperator),
    FixedVariable(usize),
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

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct FixedPointOperator {
    pub is_greatest: bool,
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
        assert!(index < property.arena.len());
        Self { property, index }
    }

    pub fn children(&self) -> Vec<Subproperty> {
        let ty = &self.property.get_by_index(self.index).ty;

        let indices: Vec<usize> = match ty {
            PropertyType::Const(_) => Vec::new(),
            PropertyType::Atomic(_) => Vec::new(),
            PropertyType::Negation(inner) => vec![*inner],
            PropertyType::BiLogic(op) => vec![op.a, op.b],
            PropertyType::Next(op) => vec![op.inner],
            PropertyType::FixedPoint(op) => {
                vec![op.inner]
            }
            PropertyType::FixedVariable(_) => Vec::new(),
        };

        indices
            .into_iter()
            .map(|index| Subproperty::new(self.property.clone(), index))
            .collect()
    }
}
