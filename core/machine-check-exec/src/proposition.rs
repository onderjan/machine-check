use machine_check_common::ExecError;

mod enf;
mod parser;
mod pnf;

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
pub struct Literal {
    complementary: bool,
    name: String,
}

impl Literal {
    pub fn new(name: String) -> Literal {
        Literal {
            complementary: false,
            name,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn is_complementary(&self) -> bool {
        self.complementary
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