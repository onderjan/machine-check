use std::fmt::Debug;

use crate::{
    iir::{
        interpretation::{IAbstractValue, IRefinementValue, Interpretation},
        variable::IVarId,
    },
    ir_common::{IrMckBinaryOp, IrMckUnaryOp},
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IMckUnary {
    pub op: IrMckUnaryOp,
    pub operand: IVarId,
}

impl IMckUnary {
    pub(super) fn forward_interpret(
        &self,
        abstr: &Interpretation<IAbstractValue>,
    ) -> IAbstractValue {
        let operand = abstr.value(self.operand).clone();
        match self.op {
            IrMckUnaryOp::Not => mck::forward::Bitwise::bit_not(operand),
            IrMckUnaryOp::Neg => mck::forward::HwArith::arith_neg(operand),
        }
    }

    pub(super) fn backward_interpret(
        &self,
        abstr: &Interpretation<IAbstractValue>,
        refin: &mut Interpretation<IRefinementValue>,
        later: IRefinementValue,
    ) {
        let operand = abstr.value(self.operand).clone();
        let earlier = match self.op {
            IrMckUnaryOp::Not => mck::backward::Bitwise::bit_not((operand,), later).0,

            IrMckUnaryOp::Neg => mck::backward::HwArith::arith_neg((operand,), later).0,
        };

        refin.join_value(self.operand, earlier);
    }
}

impl Debug for IMckUnary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({:?})", self.op, self.operand)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IMckBinary {
    pub op: IrMckBinaryOp,
    pub a: IVarId,
    pub b: IVarId,
}

impl IMckBinary {
    pub(super) fn forward_interpret(
        &self,
        inter: &Interpretation<IAbstractValue>,
    ) -> IAbstractValue {
        let a = inter.value(self.a).clone();
        let b = inter.value(self.b).clone();

        match self.op {
            IrMckBinaryOp::BitAnd => mck::forward::Bitwise::bit_and(a, b),
            IrMckBinaryOp::BitOr => mck::forward::Bitwise::bit_or(a, b),
            IrMckBinaryOp::BitXor => mck::forward::Bitwise::bit_xor(a, b),

            IrMckBinaryOp::LogicShl => mck::forward::HwShift::logic_shl(a, b),
            IrMckBinaryOp::LogicShr => mck::forward::HwShift::logic_shr(a, b),
            IrMckBinaryOp::ArithShr => mck::forward::HwShift::arith_shr(a, b),

            IrMckBinaryOp::Add => mck::forward::HwArith::add(a, b),
            IrMckBinaryOp::Sub => mck::forward::HwArith::sub(a, b),
            IrMckBinaryOp::Mul => mck::forward::HwArith::mul(a, b),
            IrMckBinaryOp::Udiv => mck::forward::HwArith::udiv(a, b),
            IrMckBinaryOp::Urem => mck::forward::HwArith::urem(a, b),
            IrMckBinaryOp::Sdiv => mck::forward::HwArith::sdiv(a, b),
            IrMckBinaryOp::Srem => mck::forward::HwArith::srem(a, b),

            IrMckBinaryOp::Eq => mck::forward::TypedEq::eq(a, b),
            IrMckBinaryOp::Ne => mck::forward::TypedEq::ne(a, b),

            IrMckBinaryOp::Ult => mck::forward::TypedCmp::ult(a, b),
            IrMckBinaryOp::Ule => mck::forward::TypedCmp::ule(a, b),
            IrMckBinaryOp::Slt => mck::forward::TypedCmp::slt(a, b),
            IrMckBinaryOp::Sle => mck::forward::TypedCmp::sle(a, b),
        }
    }

    pub(super) fn backward_interpret(
        &self,
        abstr: &Interpretation<IAbstractValue>,
        refin: &mut Interpretation<IRefinementValue>,
        later: IRefinementValue,
    ) {
        let a = abstr.value(self.a).clone();
        let b = abstr.value(self.b).clone();

        let (earlier_a, earlier_b) = match self.op {
            IrMckBinaryOp::BitAnd => mck::backward::Bitwise::bit_and((a, b), later),
            IrMckBinaryOp::BitOr => mck::backward::Bitwise::bit_or((a, b), later),
            IrMckBinaryOp::BitXor => mck::backward::Bitwise::bit_xor((a, b), later),

            IrMckBinaryOp::LogicShl => mck::backward::HwShift::logic_shl((a, b), later),
            IrMckBinaryOp::LogicShr => mck::backward::HwShift::logic_shr((a, b), later),
            IrMckBinaryOp::ArithShr => mck::backward::HwShift::arith_shr((a, b), later),

            IrMckBinaryOp::Add => mck::backward::HwArith::add((a, b), later),
            IrMckBinaryOp::Sub => mck::backward::HwArith::sub((a, b), later),
            IrMckBinaryOp::Mul => mck::backward::HwArith::mul((a, b), later),
            IrMckBinaryOp::Udiv => mck::backward::HwArith::udiv((a, b), later),
            IrMckBinaryOp::Urem => mck::backward::HwArith::urem((a, b), later),
            IrMckBinaryOp::Sdiv => mck::backward::HwArith::sdiv((a, b), later),
            IrMckBinaryOp::Srem => mck::backward::HwArith::srem((a, b), later),

            IrMckBinaryOp::Eq => mck::backward::TypedEq::eq((a, b), later),
            IrMckBinaryOp::Ne => mck::backward::TypedEq::ne((a, b), later),

            IrMckBinaryOp::Ult => mck::backward::TypedCmp::ult((a, b), later),
            IrMckBinaryOp::Ule => mck::backward::TypedCmp::ule((a, b), later),
            IrMckBinaryOp::Slt => mck::backward::TypedCmp::slt((a, b), later),
            IrMckBinaryOp::Sle => mck::backward::TypedCmp::sle((a, b), later),
        };

        refin.join_value(self.a, earlier_a);
        refin.join_value(self.b, earlier_b);
    }
}

impl Debug for IMckBinary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({:?}, {:?})", self.op, self.a, self.b)
    }
}
