use super::id::FlippableNid;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Btor2ExtOp {
    pub a: FlippableNid,
    pub extension_size: usize,
    pub signed: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Btor2SliceOp {
    pub a: FlippableNid,
    pub low_bit: usize,
    pub high_bit: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Btor2UniOpType {
    Not,
    Inc,
    Dec,
    Neg,
    Redand,
    Redor,
    Redxor,
}

#[derive(Debug, Clone)]
pub struct Btor2UniOp {
    pub op_type: Btor2UniOpType,
    pub a: FlippableNid,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Btor2BiOpType {
    // Boolean
    Iff,
    Implies,
    // (dis)equality
    Eq,
    Neq,
    // (un)signed equality
    Sgt,
    Ugt,
    Sgte,
    Ugte,
    Slt,
    Ult,
    Slte,
    Ulte,
    // bitwise
    And,
    Nand,
    Nor,
    Or,
    Xnor,
    Xor,
    // rotate
    Rol,
    Ror,
    // shift
    Sll,
    Sra,
    Srl,
    // arithmetic
    Add,
    Mul,
    Sdiv,
    Udiv,
    Smod,
    Srem,
    Urem,
    Sub,
    // overflow
    Saddo,
    Uaddo,
    Sdivo,
    Udivo,
    Smulo,
    Umulo,
    Ssubo,
    Usubo,
    // concatenation
    Concat,
    // array read
    Read,
}

impl TryFrom<&str> for Btor2BiOpType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            // Boolean
            "iff" => Ok(Btor2BiOpType::Iff),
            "implies" => Ok(Btor2BiOpType::Implies),
            // (dis)equality
            "eq" => Ok(Btor2BiOpType::Eq),
            "neq" => Ok(Btor2BiOpType::Neq),
            // (un)signed equality
            "sgt" => Ok(Btor2BiOpType::Sgt),
            "ugt" => Ok(Btor2BiOpType::Ugt),
            "sgte" => Ok(Btor2BiOpType::Sgte),
            "ugte" => Ok(Btor2BiOpType::Ugte),
            "slt" => Ok(Btor2BiOpType::Slt),
            "ult" => Ok(Btor2BiOpType::Ult),
            "slte" => Ok(Btor2BiOpType::Slte),
            "ulte" => Ok(Btor2BiOpType::Ulte),
            // bitwise
            "and" => Ok(Btor2BiOpType::And),
            "nand" => Ok(Btor2BiOpType::Nand),
            "nor" => Ok(Btor2BiOpType::Nor),
            "or" => Ok(Btor2BiOpType::Or),
            "xnor" => Ok(Btor2BiOpType::Xnor),
            "xor" => Ok(Btor2BiOpType::Xor),
            // rotate
            "rol" => Ok(Btor2BiOpType::Rol),
            "ror" => Ok(Btor2BiOpType::Ror),
            // shift
            "sll" => Ok(Btor2BiOpType::Sll),
            "sra" => Ok(Btor2BiOpType::Sra),
            "srl" => Ok(Btor2BiOpType::Srl),
            // arithmetic
            "add" => Ok(Btor2BiOpType::Add),
            "mul" => Ok(Btor2BiOpType::Mul),
            "sdiv" => Ok(Btor2BiOpType::Sdiv),
            "udiv" => Ok(Btor2BiOpType::Udiv),
            "smod" => Ok(Btor2BiOpType::Smod),
            "srem" => Ok(Btor2BiOpType::Srem),
            "urem" => Ok(Btor2BiOpType::Urem),
            "sub" => Ok(Btor2BiOpType::Sub),
            // overflow
            "saddo" => Ok(Btor2BiOpType::Saddo),
            "uaddo" => Ok(Btor2BiOpType::Uaddo),
            "sdivo" => Ok(Btor2BiOpType::Sdivo),
            "udivo" => Ok(Btor2BiOpType::Udivo),
            "smulo" => Ok(Btor2BiOpType::Smulo),
            "umulo" => Ok(Btor2BiOpType::Umulo),
            "ssubo" => Ok(Btor2BiOpType::Ssubo),
            "usubo" => Ok(Btor2BiOpType::Usubo),
            // concatenation
            "concat" => Ok(Btor2BiOpType::Concat),
            // array read
            "read" => Ok(Btor2BiOpType::Read),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Btor2BiOp {
    pub op_type: Btor2BiOpType,
    pub a: FlippableNid,
    pub b: FlippableNid,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Btor2TriOpType {
    // if-then-else
    Ite,
    // array write
    Write,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Btor2TriOp {
    pub op_type: Btor2TriOpType,
    pub a: FlippableNid,
    pub b: FlippableNid,
    pub c: FlippableNid,
}
