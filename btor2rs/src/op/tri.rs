use crate::rref::Rref;

// derive Btor2 string representations, which are lower-case
#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum TriOpType {
    // if-then-else
    Ite,
    // array write
    Write,
}

#[derive(Debug, Clone)]
pub struct TriOp {
    pub op_type: TriOpType,
    pub a: Rref,
    pub b: Rref,
    pub c: Rref,
}
