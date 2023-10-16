use crate::id::{Rnid, Sid};

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
    pub sid: Sid,
    pub ty: UniOpType,
    pub a: Rnid,
}

#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
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

#[derive(Debug, Clone)]
pub struct BiOp {
    pub sid: Sid,
    pub ty: BiOpType,
    pub a: Rnid,
    pub b: Rnid,
}

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
    pub sid: Sid,
    pub ty: TriOpType,
    pub a: Rnid,
    pub b: Rnid,
    pub c: Rnid,
}

#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum ExtOpType {
    Sext,
    Uext,
}

#[derive(Debug, Clone)]
pub struct ExtOp {
    pub sid: Sid,
    pub ty: ExtOpType,
    pub a: Rnid,
    pub length: u32,
}

#[derive(Debug, Clone)]
pub struct SliceOp {
    pub sid: Sid,
    pub a: Rnid,
    pub upper_bit: u32,
    pub lower_bit: u32,
}
