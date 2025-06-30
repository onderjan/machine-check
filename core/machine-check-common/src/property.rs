//! Computation Tree Logic properties.
use std::{fmt::Display, sync::Arc};

use crate::{ExecError, Signedness};
use serde::{Deserialize, Serialize};

mod enf;
mod misc;
mod parser;
mod pnf;

/// A Computation Tree Logic property.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Property {
    Const(bool),
    Atomic(AtomicProperty),
    Negation(UniOperator),
    Or(BiOperator),
    And(BiOperator),
    E(TemporalOperator),
    A(TemporalOperator),
    LeastFixedPoint(FixedPointOperator),
    GreatestFixedPoint(FixedPointOperator),
    FixedPointVariable(FixedPointVariable),
}

/// A temporal operator within a CTL path quantifier.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum TemporalOperator {
    X(UniOperator),
    F(OperatorF),
    G(OperatorG),
    U(OperatorU),
    R(OperatorR),
}

/// A fixed-point operator.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct FixedPointOperator {
    pub variable: FixedPointVariable,
    pub inner: Box<Property>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct FixedPointVariable {
    pub id: u64,
    pub name: Arc<String>,
}

impl Property {
    pub fn parse(prop_str: &str) -> Result<Property, ExecError> {
        parser::parse(prop_str)
    }

    pub fn inherent() -> Property {
        let not_panicking = AtomicProperty::new(
            ValueExpression {
                name: String::from("__panic"),
                index: None,
                forced_signedness: Signedness::None,
            },
            crate::property::ComparisonType::Eq,
            0,
        );
        Property::A(TemporalOperator::G(OperatorG(Box::new(Property::Atomic(
            not_panicking,
        )))))
    }

    pub fn children(&self) -> Vec<Property> {
        match self {
            Property::Const(_) => Vec::new(),
            Property::Atomic(_) => Vec::new(),
            Property::Negation(prop_uni) => vec![*prop_uni.0.clone()],
            Property::Or(prop_bi) => vec![*prop_bi.a.clone(), *prop_bi.b.clone()],
            Property::And(prop_bi) => vec![*prop_bi.a.clone(), *prop_bi.b.clone()],
            Property::E(prop_temp) => prop_temp.children(),
            Property::A(prop_temp) => prop_temp.children(),
            Property::LeastFixedPoint(fixed_point_operator) => {
                vec![*fixed_point_operator.inner.clone()]
            }
            Property::GreatestFixedPoint(fixed_point_operator) => {
                vec![*fixed_point_operator.inner.clone()]
            }
            Property::FixedPointVariable(_) => Vec::new(),
        }
    }
}

impl TemporalOperator {
    pub fn children(&self) -> Vec<Property> {
        match self {
            TemporalOperator::X(prop_uni) => {
                vec![*prop_uni.0.clone()]
            }
            TemporalOperator::F(prop_f) => {
                vec![*prop_f.0.clone()]
            }
            TemporalOperator::G(prop_g) => {
                vec![*prop_g.0.clone()]
            }
            TemporalOperator::U(prop_u) => {
                vec![*prop_u.hold.clone(), *prop_u.until.clone()]
            }
            TemporalOperator::R(prop_r) => {
                vec![*prop_r.releaser.clone(), *prop_r.releasee.clone()]
            }
        }
    }
}

impl Display for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Property::Const(value) => {
                write!(f, "{}", value)
            }
            Property::Atomic(literal) => {
                write!(f, "{}", literal)
            }
            Property::Negation(prop_uni) => {
                write!(f, "!({})", prop_uni.0)
            }
            Property::Or(prop_bi) => write_logic_bi(f, prop_bi, "||"),
            Property::And(prop_bi) => write_logic_bi(f, prop_bi, "&&"),
            Property::E(prop_temp) => {
                write!(f, "E{}", prop_temp)
            }
            Property::A(prop_temp) => {
                write!(f, "A{}", prop_temp)
            }
            Property::LeastFixedPoint(fixed_point_operator) => {
                write!(
                    f,
                    "lfp![{},{}]",
                    fixed_point_operator.variable.name, fixed_point_operator.inner
                )
            }
            Property::GreatestFixedPoint(fixed_point_operator) => write!(
                f,
                "gfp![{},{}]",
                fixed_point_operator.variable.name, fixed_point_operator.inner
            ),
            Property::FixedPointVariable(var) => write!(f, "{}", var.name),
        }
    }
}

fn write_logic_bi(
    f: &mut std::fmt::Formatter<'_>,
    prop_bi: &BiOperator,
    op_str: &str,
) -> std::fmt::Result {
    // Make sure the inner and / or properties are in parentheses so the display is unambiguous.
    let write_inner_prop = |f: &mut std::fmt::Formatter<'_>, prop: &Property| {
        if matches!(prop, Property::And(_) | Property::Or(_)) {
            write!(f, "({})", prop)
        } else {
            write!(f, "{}", prop)
        }
    };

    write_inner_prop(f, &prop_bi.a)?;
    write!(f, " {} ", op_str)?;
    write_inner_prop(f, &prop_bi.b)
}

impl Display for TemporalOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemporalOperator::X(prop_uni) => {
                write!(f, "X![{}]", prop_uni.0)
            }
            TemporalOperator::F(prop_f) => {
                write!(f, "F![{}]", prop_f.0)
            }
            TemporalOperator::G(prop_g) => {
                write!(f, "G![{}]", prop_g.0)
            }
            TemporalOperator::U(prop_u) => {
                write!(f, "U![{}, {}]", prop_u.hold, prop_u.until)
            }
            TemporalOperator::R(prop_r) => {
                write!(f, "R![{}, {}]", prop_r.releaser, prop_r.releasee)
            }
        }
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

/// A field name, potentially with indexing and forced signedness.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ValueExpression {
    name: String,
    index: Option<u64>,
    forced_signedness: Signedness,
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

/// An atomic property of Computation Tree Logic.
///
/// In our case, this is usually a field compared to a number.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct AtomicProperty {
    complementary: bool,
    left: ValueExpression,
    comparison_type: ComparisonType,
    right_number: i64,
}

impl AtomicProperty {
    pub fn new(
        left: ValueExpression,
        comparison_type: ComparisonType,
        right_number: i64,
    ) -> AtomicProperty {
        AtomicProperty {
            complementary: false,
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

    pub fn is_complementary(&self) -> bool {
        self.complementary
    }
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

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct UniOperator(pub Box<Property>);

impl UniOperator {
    pub fn new(prop: Property) -> Self {
        UniOperator(Box::new(prop))
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct BiOperator {
    pub a: Box<Property>,
    pub b: Box<Property>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct OperatorF(pub Box<Property>);

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct OperatorG(pub Box<Property>);

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct OperatorU {
    pub hold: Box<Property>,
    pub until: Box<Property>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct OperatorR {
    pub releaser: Box<Property>,
    pub releasee: Box<Property>,
}
