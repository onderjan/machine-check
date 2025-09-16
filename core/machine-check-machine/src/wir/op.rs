use machine_check_common::ir_common::{IrMckBinaryOp, IrMckUnaryOp, IrStdBinaryOp, IrStdUnaryOp};

use super::WIdent;

#[derive(Clone, Debug, Hash)]
pub struct WStdUnary {
    pub op: IrStdUnaryOp,
    pub operand: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub struct WStdBinary {
    pub op: IrStdBinaryOp,
    pub a: WIdent,
    pub b: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub struct WMckUnary {
    pub op: IrMckUnaryOp,
    pub operand: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub struct WMckBinary {
    pub op: IrMckBinaryOp,
    pub a: WIdent,
    pub b: WIdent,
}
