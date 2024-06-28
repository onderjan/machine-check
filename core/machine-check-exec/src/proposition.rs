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
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum InequalityType {
    Lt,
    Le,
    Gt,
    Ge,
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
    pub hold: Box<Proposition>,
    pub release: Box<Proposition>,
}
