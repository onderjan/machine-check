//! Btor2 operation nodes.
//!
//! Although the operations are normal nodes in a sense,
//! they are in a separate module for clarity.

use crate::id::{Rnid, Sid};

/// Unary operation type.
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

/// Unary operation node.
#[derive(Debug, Clone)]
pub struct UniOp {
    /// Result sort.
    pub sid: Sid,
    /// Type of operation.
    pub ty: UniOpType,
    /// Operand right-side node id.
    pub a: Rnid,
}

/// Binary operation type.
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

/// Binary operation node.
#[derive(Debug, Clone)]
pub struct BiOp {
    /// Result sort.
    pub sid: Sid,
    /// Operation type.
    pub ty: BiOpType,
    /// First operand right-side node id.
    pub a: Rnid,
    /// Second operand right-side node id.
    pub b: Rnid,
}

/// Ternary operation type.
#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum TriOpType {
    // if-then-else
    Ite,
    // array write
    Write,
}

/// Ternary operation node.
#[derive(Debug, Clone)]
pub struct TriOp {
    /// Result sort.
    pub sid: Sid,
    /// Operation type.
    pub ty: TriOpType,
    /// First operand right-side node id.
    pub a: Rnid,
    /// Second operand right-side node id.
    pub b: Rnid,
    /// Third operand right-side node id.
    pub c: Rnid,
}

/// Extension operation type.
#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum ExtOpType {
    Sext,
    Uext,
}

/// Extension operation node.
#[derive(Debug, Clone)]
pub struct ExtOp {
    /// Result sort.
    pub sid: Sid,
    /// Operation type.
    pub ty: ExtOpType,
    /// Operand right-side node id.
    pub a: Rnid,
    /// Length of extension.
    pub length: u32,
}

/// Slice operation node.
#[derive(Debug, Clone)]
pub struct SliceOp {
    /// Result sort.
    pub sid: Sid,
    /// Operand right-side node id.
    pub a: Rnid,
    /// Upper bit of slice (inclusive).
    ///
    /// Guaranteed to be greater or equal to lower bit after parsing.
    pub upper_bit: u32,
    /// Lower bit of slice (inclusive).
    ///
    /// Guaranteed to be lesser or equal to upper bit after parsing.
    pub lower_bit: u32,
}
