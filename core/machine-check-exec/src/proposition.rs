use std::fmt::Display;

use machine_check_common::ExecError;

mod enf;
mod misc;
mod parser;
mod pnf;

/// CTL proposition.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Proposition {
    Const(bool),
    Literal(Literal),
    Negation(PropUni),
    Or(PropBi),
    And(PropBi),
    E(PropTemp),
    A(PropTemp),
}

/// Temporal operator within CTL path quantifier.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PropTemp {
    X(PropUni),
    F(PropF),
    G(PropG),
    U(PropU),
    R(PropR),
}

impl Proposition {
    pub fn parse(prop_str: &str) -> Result<Proposition, ExecError> {
        parser::parse(prop_str)
    }

    pub fn inherent() -> Proposition {
        Proposition::A(PropTemp::G(PropG(Box::new(Proposition::Literal(
            Literal::new(
                String::from("__panic"),
                crate::proposition::ComparisonType::Eq,
                0,
                None,
            ),
        )))))
    }

    pub fn children(&self) -> Vec<Proposition> {
        match self {
            Proposition::Const(_) => Vec::new(),
            Proposition::Literal(_) => Vec::new(),
            Proposition::Negation(prop_uni) => vec![*prop_uni.0.clone()],
            Proposition::Or(prop_bi) => vec![*prop_bi.a.clone(), *prop_bi.b.clone()],
            Proposition::And(prop_bi) => vec![*prop_bi.a.clone(), *prop_bi.b.clone()],
            Proposition::E(prop_temp) => prop_temp.children(),
            Proposition::A(prop_temp) => prop_temp.children(),
        }
    }
}

impl PropTemp {
    pub fn children(&self) -> Vec<Proposition> {
        match self {
            PropTemp::X(prop_uni) => {
                vec![*prop_uni.0.clone()]
            }
            PropTemp::F(prop_f) => {
                vec![*prop_f.0.clone()]
            }
            PropTemp::G(prop_g) => {
                vec![*prop_g.0.clone()]
            }
            PropTemp::U(prop_u) => {
                vec![*prop_u.hold.clone(), *prop_u.until.clone()]
            }
            PropTemp::R(prop_r) => {
                vec![*prop_r.releaser.clone(), *prop_r.releasee.clone()]
            }
        }
    }
}

impl Display for Proposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Proposition::Const(value) => {
                write!(f, "{}", value)
            }
            Proposition::Literal(literal) => {
                write!(f, "{}", literal)
            }
            Proposition::Negation(prop_uni) => {
                write!(f, "!({})", prop_uni.0)
            }
            Proposition::Or(prop_bi) => {
                write!(f, "({}) | ({})", prop_bi.a, prop_bi.b)
            }
            Proposition::And(prop_bi) => {
                write!(f, "({}) & ({})", prop_bi.a, prop_bi.b)
            }
            Proposition::E(prop_temp) => {
                write!(f, "E{}", prop_temp)
            }
            Proposition::A(prop_temp) => {
                write!(f, "A{}", prop_temp)
            }
        }
    }
}

impl Display for PropTemp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropTemp::X(prop_uni) => {
                write!(f, "X[{}]", prop_uni.0)
            }
            PropTemp::F(prop_f) => {
                write!(f, "F[{}]", prop_f.0)
            }
            PropTemp::G(prop_g) => {
                write!(f, "G[{}]", prop_g.0)
            }
            PropTemp::U(prop_u) => {
                write!(f, "[{}]U[{}]", prop_u.hold, prop_u.until)
            }
            PropTemp::R(prop_r) => {
                write!(f, "[{}]R[{}]", prop_r.releaser, prop_r.releasee)
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ComparisonType {
    Eq,
    Neq,
    Unsigned(InequalityType),
    Signed(InequalityType),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PropUni(pub Box<Proposition>);

impl PropUni {
    pub fn new(prop: Proposition) -> Self {
        PropUni(Box::new(prop))
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PropBi {
    pub a: Box<Proposition>,
    pub b: Box<Proposition>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PropF(pub Box<Proposition>);

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PropG(pub Box<Proposition>);

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PropU {
    pub hold: Box<Proposition>,
    pub until: Box<Proposition>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PropR {
    pub releaser: Box<Proposition>,
    pub releasee: Box<Proposition>,
}
