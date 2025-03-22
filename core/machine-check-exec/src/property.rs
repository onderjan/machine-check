use std::fmt::Display;

use machine_check_common::ExecError;
use serde::{Deserialize, Serialize};

mod enf;
mod misc;
mod parser;
mod pnf;

/// CTL proposition.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Property {
    Const(bool),
    Literal(Literal),
    Negation(UniOperator),
    Or(BiOperator),
    And(BiOperator),
    E(TemporalOperator),
    A(TemporalOperator),
}

/// Temporal operator within CTL path quantifier.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum TemporalOperator {
    X(UniOperator),
    F(OperatorF),
    G(OperatorG),
    U(OperatorU),
    R(OperatorR),
}

impl Property {
    pub fn parse(prop_str: &str) -> Result<Property, ExecError> {
        parser::parse(prop_str)
    }

    pub fn inherent() -> Property {
        Property::A(TemporalOperator::G(OperatorG(Box::new(Property::Literal(
            Literal::new(
                String::from("__panic"),
                crate::property::ComparisonType::Eq,
                0,
                None,
            ),
        )))))
    }

    pub fn children(&self) -> Vec<Property> {
        match self {
            Property::Const(_) => Vec::new(),
            Property::Literal(_) => Vec::new(),
            Property::Negation(prop_uni) => vec![*prop_uni.0.clone()],
            Property::Or(prop_bi) => vec![*prop_bi.a.clone(), *prop_bi.b.clone()],
            Property::And(prop_bi) => vec![*prop_bi.a.clone(), *prop_bi.b.clone()],
            Property::E(prop_temp) => prop_temp.children(),
            Property::A(prop_temp) => prop_temp.children(),
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
            Property::Literal(literal) => {
                write!(f, "{}", literal)
            }
            Property::Negation(prop_uni) => {
                write!(f, "!({})", prop_uni.0)
            }
            Property::Or(prop_bi) => {
                write!(f, "({}) | ({})", prop_bi.a, prop_bi.b)
            }
            Property::And(prop_bi) => {
                write!(f, "({}) & ({})", prop_bi.a, prop_bi.b)
            }
            Property::E(prop_temp) => {
                write!(f, "E{}", prop_temp)
            }
            Property::A(prop_temp) => {
                write!(f, "A{}", prop_temp)
            }
        }
    }
}

impl Display for TemporalOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemporalOperator::X(prop_uni) => {
                write!(f, "X[{}]", prop_uni.0)
            }
            TemporalOperator::F(prop_f) => {
                write!(f, "F[{}]", prop_f.0)
            }
            TemporalOperator::G(prop_g) => {
                write!(f, "G[{}]", prop_g.0)
            }
            TemporalOperator::U(prop_u) => {
                write!(f, "[{}]U[{}]", prop_u.hold, prop_u.until)
            }
            TemporalOperator::R(prop_r) => {
                write!(f, "[{}]R[{}]", prop_r.releaser, prop_r.releasee)
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum InequalityType {
    Lt,
    Le,
    Gt,
    Ge,
}

impl Display for InequalityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inequality_str = match self {
            InequalityType::Lt => "<",
            InequalityType::Le => "<=",
            InequalityType::Gt => ">",
            InequalityType::Ge => ">=",
        };

        write!(f, "{}", inequality_str)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ComparisonType {
    Eq,
    Neq,
    Unsigned(InequalityType),
    Signed(InequalityType),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Literal {
    complementary: bool,
    left_name: String,
    comparison_type: ComparisonType,
    right_number: u64,
    index: Option<u64>,
}

impl Literal {
    pub fn new(
        left_name: String,
        comparison_type: ComparisonType,
        right_number: u64,
        index: Option<u64>,
    ) -> Literal {
        Literal {
            complementary: false,
            left_name,
            comparison_type,
            right_number,
            index,
        }
    }

    pub fn name(&self) -> &str {
        self.left_name.as_str()
    }

    pub fn comparison_type(&self) -> &ComparisonType {
        &self.comparison_type
    }

    pub fn right_number_unsigned(&self) -> u64 {
        self.right_number
    }

    pub fn right_number_signed(&self) -> i64 {
        self.right_number as i64
    }

    pub fn is_complementary(&self) -> bool {
        self.complementary
    }

    pub fn index(&self) -> Option<u64> {
        self.index
    }

    fn left_string(&self) -> String {
        if let Some(index) = self.index {
            format!("{}[{}]", self.left_name, index)
        } else {
            self.left_name.clone()
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.comparison_type {
            ComparisonType::Eq => write!(f, "{} == {}", &self.left_string(), self.right_number),
            ComparisonType::Neq => write!(f, "{} != {}", &self.left_string(), self.right_number),
            ComparisonType::Unsigned(inequality_type) => {
                write!(
                    f,
                    "unsigned({}) {} {}",
                    &self.left_string(),
                    &inequality_type.to_string(),
                    self.right_number
                )
            }
            ComparisonType::Signed(inequality_type) => {
                write!(
                    f,
                    "signed({}) {} {}",
                    &self.left_string(),
                    &inequality_type.to_string(),
                    self.right_number
                )
            }
        }
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
