use super::id::FlippableNid;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExtOp {
    pub a: FlippableNid,
    pub extension_size: usize,
    pub signed: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SliceOp {
    pub a: FlippableNid,
    pub low_bit: usize,
    pub high_bit: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum UniOpType {
    Not,
    Inc,
    Dec,
    Neg,
    Redand,
    Redor,
    Redxor,
}

#[derive(Debug, Clone)]
pub struct UniOp {
    pub op_type: UniOpType,
    pub a: FlippableNid,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum BiOpType {
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

impl TryFrom<&str> for BiOpType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            // Boolean
            "iff" => Ok(BiOpType::Iff),
            "implies" => Ok(BiOpType::Implies),
            // (dis)equality
            "eq" => Ok(BiOpType::Eq),
            "neq" => Ok(BiOpType::Neq),
            // (un)signed equality
            "sgt" => Ok(BiOpType::Sgt),
            "ugt" => Ok(BiOpType::Ugt),
            "sgte" => Ok(BiOpType::Sgte),
            "ugte" => Ok(BiOpType::Ugte),
            "slt" => Ok(BiOpType::Slt),
            "ult" => Ok(BiOpType::Ult),
            "slte" => Ok(BiOpType::Slte),
            "ulte" => Ok(BiOpType::Ulte),
            // bitwise
            "and" => Ok(BiOpType::And),
            "nand" => Ok(BiOpType::Nand),
            "nor" => Ok(BiOpType::Nor),
            "or" => Ok(BiOpType::Or),
            "xnor" => Ok(BiOpType::Xnor),
            "xor" => Ok(BiOpType::Xor),
            // rotate
            "rol" => Ok(BiOpType::Rol),
            "ror" => Ok(BiOpType::Ror),
            // shift
            "sll" => Ok(BiOpType::Sll),
            "sra" => Ok(BiOpType::Sra),
            "srl" => Ok(BiOpType::Srl),
            // arithmetic
            "add" => Ok(BiOpType::Add),
            "mul" => Ok(BiOpType::Mul),
            "sdiv" => Ok(BiOpType::Sdiv),
            "udiv" => Ok(BiOpType::Udiv),
            "smod" => Ok(BiOpType::Smod),
            "srem" => Ok(BiOpType::Srem),
            "urem" => Ok(BiOpType::Urem),
            "sub" => Ok(BiOpType::Sub),
            // overflow
            "saddo" => Ok(BiOpType::Saddo),
            "uaddo" => Ok(BiOpType::Uaddo),
            "sdivo" => Ok(BiOpType::Sdivo),
            "udivo" => Ok(BiOpType::Udivo),
            "smulo" => Ok(BiOpType::Smulo),
            "umulo" => Ok(BiOpType::Umulo),
            "ssubo" => Ok(BiOpType::Ssubo),
            "usubo" => Ok(BiOpType::Usubo),
            // concatenation
            "concat" => Ok(BiOpType::Concat),
            // array read
            "read" => Ok(BiOpType::Read),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BiOp {
    pub op_type: BiOpType,
    pub a: FlippableNid,
    pub b: FlippableNid,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TriOpType {
    // if-then-else
    Ite,
    // array write
    Write,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TriOp {
    pub op_type: TriOpType,
    pub a: FlippableNid,
    pub b: FlippableNid,
    pub c: FlippableNid,
}
