use std::fmt::Display;

use crate::{
    property::{AtomicProperty, ValueExpression},
    Signedness,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Property {
    Const(bool),
    Atomic(AtomicProperty),
    Negation(Box<Property>),
    BiLogicOperator(BiLogicOperator),
    CtlOperator(CtlOperator),
    LeastFixedPoint(FixedPointOperator),
    GreatestFixedPoint(FixedPointOperator),
    FixedPointVariable(String),
}

impl Property {
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
        let not_panicking = Box::new(Property::Atomic(not_panicking));

        let g_operator = TemporalOperator::G(OperatorG(not_panicking));
        Property::CtlOperator(CtlOperator {
            is_universal: true,
            temporal: g_operator,
        })
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BiLogicOperator {
    pub is_and: bool,
    pub a: Box<Property>,
    pub b: Box<Property>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CtlOperator {
    pub is_universal: bool,
    pub temporal: TemporalOperator,
}

/// A temporal operator within a CTL path quantifier.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TemporalOperator {
    X(Box<Property>),
    F(OperatorF),
    G(OperatorG),
    U(OperatorU),
    R(OperatorR),
}

/// A fixed-point operator.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FixedPointOperator {
    pub variable: String,
    pub inner: Box<Property>,
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
                write!(f, "!({})", *prop_uni)
            }
            Property::BiLogicOperator(op) => write_logic_bi(f, op),
            Property::CtlOperator(op) => {
                let quantifier_letter = if op.is_universal { 'A' } else { 'E' };
                write!(f, "{}{}", quantifier_letter, op.temporal)
            }
            Property::LeastFixedPoint(fixed_point_operator) => {
                write!(
                    f,
                    "lfp![{}, {}]",
                    fixed_point_operator.variable, fixed_point_operator.inner
                )
            }
            Property::GreatestFixedPoint(fixed_point_operator) => write!(
                f,
                "gfp![{}, {}]",
                fixed_point_operator.variable, fixed_point_operator.inner
            ),
            Property::FixedPointVariable(var) => write!(f, "{}", var),
        }
    }
}

fn write_logic_bi(f: &mut std::fmt::Formatter<'_>, op: &BiLogicOperator) -> std::fmt::Result {
    let op_str = if op.is_and { "&&" } else { "||" };
    // Make sure the inner and / or properties are in parentheses so the display is unambiguous.
    let write_inner_prop = |f: &mut std::fmt::Formatter<'_>, prop: &Property| {
        if matches!(prop, Property::BiLogicOperator(..)) {
            write!(f, "({})", prop)
        } else {
            write!(f, "{}", prop)
        }
    };

    write_inner_prop(f, &op.a)?;
    write!(f, " {} ", op_str)?;
    write_inner_prop(f, &op.b)
}

impl Display for TemporalOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemporalOperator::X(prop_uni) => {
                write!(f, "X![{}]", *prop_uni)
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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct OperatorF(pub Box<Property>);

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct OperatorG(pub Box<Property>);

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct OperatorU {
    pub hold: Box<Property>,
    pub until: Box<Property>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct OperatorR {
    pub releaser: Box<Property>,
    pub releasee: Box<Property>,
}
