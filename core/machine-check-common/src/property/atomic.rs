use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::Signedness;

/// A field name, potentially with indexing and forced signedness.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ValueExpression {
    pub(crate) name: String,
    pub(crate) index: Option<u64>,
    pub(crate) forced_signedness: Signedness,
}

impl ValueExpression {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn index(&self) -> Option<u64> {
        self.index
    }

    pub fn forced_signedness(&self) -> Signedness {
        self.forced_signedness
    }
}

/// An atomic property of Computation Tree Logic.
///
/// In our case, this is usually a field compared to a number.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct AtomicProperty {
    pub(crate) left: ValueExpression,
    pub(crate) comparison_type: ComparisonType,
    pub(crate) right_number: i64,
}

impl AtomicProperty {
    pub fn new(
        left: ValueExpression,
        comparison_type: ComparisonType,
        right_number: i64,
    ) -> AtomicProperty {
        AtomicProperty {
            left,
            comparison_type,
            right_number,
        }
    }

    pub fn left(&self) -> &ValueExpression {
        &self.left
    }

    pub fn comparison_type(&self) -> &ComparisonType {
        &self.comparison_type
    }

    pub fn right_number_unsigned(&self) -> u64 {
        self.right_number as u64
    }

    pub fn right_number_signed(&self) -> i64 {
        self.right_number
    }
}

/// A type of comparison.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ComparisonType {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Display for AtomicProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.left, self.comparison_type, self.right_number
        )
    }
}

impl Display for ValueExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let add_closing_parenthesis = match self.forced_signedness {
            Signedness::Unsigned => {
                write!(f, "as_unsigned(")?;
                true
            }
            Signedness::Signed => {
                write!(f, "as_signed(")?;
                true
            }
            Signedness::None => false,
        };

        write!(f, "{}", self.name)?;

        if let Some(index) = self.index {
            write!(f, "[{}]", index)?;
        }
        if add_closing_parenthesis {
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl Display for ComparisonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ComparisonType::Eq => "==",
            ComparisonType::Ne => "!=",
            ComparisonType::Lt => "<",
            ComparisonType::Le => "<=",
            ComparisonType::Gt => ">",
            ComparisonType::Ge => ">=",
        };

        write!(f, "{}", str)
    }
}
