use crate::rref::Rref;

// derive Btor2 string representations, which are lower-case
#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
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
    pub a: Rref,
}

impl UniOp {
    pub fn new(op_type: UniOpType, a: Rref) -> UniOp {
        UniOp { op_type, a }
    }
}
