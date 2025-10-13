use std::fmt::Debug;

use mck::forward::Bitwise;

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
    fn forward_interpret(&self, abstr: &Interpretation<IAbstractValue>) -> IAbstractValue {
        let operand = abstr.value(self.operand).expect_bitvector();
        match self.op {
            IrMckUnaryOp::Not => IAbstractValue::Bitvector(mck::forward::Bitwise::bit_not(operand)),
            IrMckUnaryOp::Neg => {
                IAbstractValue::Bitvector(mck::forward::HwArith::arith_neg(operand))
            }
        }
    }

    fn backward_interpret(
        &self,
        abstr: &Interpretation<IAbstractValue>,
        refin: &mut Interpretation<IRefinementValue>,
        later: IRefinementValue,
    ) {
        let operand = abstr.value(self.operand).expect_bitvector();
        let earlier = match self.op {
            IrMckUnaryOp::Not => IRefinementValue::Bitvector(
                mck::backward::Bitwise::bit_not((operand,), later.expect_bitvector()).0,
            ),
            IrMckUnaryOp::Neg => IRefinementValue::Bitvector(
                mck::backward::HwArith::arith_neg((operand,), later.expect_bitvector()).0,
            ),
        };

        refin.insert_value(self.operand, earlier);
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
    fn forward_interpret(&self, inter: &Interpretation<IAbstractValue>) -> IAbstractValue {
        let a = inter.value(self.a);
        let b = inter.value(self.b);

        if let (IAbstractValue::Bool(a), IAbstractValue::Bool(b)) = (a, b) {
            let (a, b) = (*a, *b);
            return IAbstractValue::Bool(match self.op {
                IrMckBinaryOp::BitAnd => Bitwise::bit_and(a, b),
                IrMckBinaryOp::BitOr => Bitwise::bit_or(a, b),
                IrMckBinaryOp::BitXor => Bitwise::bit_xor(a, b),
                IrMckBinaryOp::LogicShl => todo!(),
                IrMckBinaryOp::LogicShr => todo!(),
                IrMckBinaryOp::ArithShr => todo!(),
                IrMckBinaryOp::Add => todo!(),
                IrMckBinaryOp::Sub => todo!(),
                IrMckBinaryOp::Mul => todo!(),
                IrMckBinaryOp::Udiv => todo!(),
                IrMckBinaryOp::Urem => todo!(),
                IrMckBinaryOp::Sdiv => todo!(),
                IrMckBinaryOp::Srem => todo!(),
                IrMckBinaryOp::Eq => todo!(),
                IrMckBinaryOp::Ne => todo!(),
                IrMckBinaryOp::Ult => todo!(),
                IrMckBinaryOp::Ule => todo!(),
                IrMckBinaryOp::Slt => todo!(),
                IrMckBinaryOp::Sle => todo!(),
            });
        }

        let a = a.expect_bitvector();
        let b = b.expect_bitvector();

        match self.op {
            IrMckBinaryOp::BitAnd => {
                IAbstractValue::Bitvector(mck::forward::Bitwise::bit_and(a, b))
            }
            IrMckBinaryOp::BitOr => IAbstractValue::Bitvector(mck::forward::Bitwise::bit_or(a, b)),
            IrMckBinaryOp::BitXor => {
                IAbstractValue::Bitvector(mck::forward::Bitwise::bit_xor(a, b))
            }
            IrMckBinaryOp::LogicShl => {
                IAbstractValue::Bitvector(mck::forward::HwShift::logic_shl(a, b))
            }
            IrMckBinaryOp::LogicShr => {
                IAbstractValue::Bitvector(mck::forward::HwShift::logic_shr(a, b))
            }
            IrMckBinaryOp::ArithShr => {
                IAbstractValue::Bitvector(mck::forward::HwShift::arith_shr(a, b))
            }
            IrMckBinaryOp::Add => IAbstractValue::Bitvector(mck::forward::HwArith::add(a, b)),
            IrMckBinaryOp::Sub => IAbstractValue::Bitvector(mck::forward::HwArith::sub(a, b)),
            IrMckBinaryOp::Mul => IAbstractValue::Bitvector(mck::forward::HwArith::mul(a, b)),
            IrMckBinaryOp::Udiv => IAbstractValue::PanicResult(mck::forward::HwArith::udiv(a, b)),
            IrMckBinaryOp::Urem => IAbstractValue::PanicResult(mck::forward::HwArith::urem(a, b)),
            IrMckBinaryOp::Sdiv => IAbstractValue::PanicResult(mck::forward::HwArith::sdiv(a, b)),
            IrMckBinaryOp::Srem => IAbstractValue::PanicResult(mck::forward::HwArith::srem(a, b)),
            IrMckBinaryOp::Eq => IAbstractValue::Bool(mck::forward::TypedEq::eq(a, b)),
            IrMckBinaryOp::Ne => IAbstractValue::Bool(mck::forward::TypedEq::ne(a, b)),
            IrMckBinaryOp::Ult => IAbstractValue::Bool(mck::forward::TypedCmp::ult(a, b)),
            IrMckBinaryOp::Ule => IAbstractValue::Bool(mck::forward::TypedCmp::ule(a, b)),
            IrMckBinaryOp::Slt => IAbstractValue::Bool(mck::forward::TypedCmp::slt(a, b)),
            IrMckBinaryOp::Sle => IAbstractValue::Bool(mck::forward::TypedCmp::sle(a, b)),
        }
    }

    fn backward_interpret(
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

        refin.insert_value(self.a, earlier_a);
        refin.insert_value(self.b, earlier_b);
    }
}

impl Debug for IMckBinary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({:?}, {:?})", self.op, self.a, self.b)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum IMckNew {
    Bitvector(u32, i128),
    // TODO: bitvector array
    //BitvectorArray(WTypeArray, WIdent),
}

impl IMckNew {
    fn forward_interpret(&self) -> IAbstractValue {
        match self {
            IMckNew::Bitvector(width, constant) => {
                let Ok(constant) = u64::try_from(*constant) else {
                    panic!("Constant outside u64");
                };
                IAbstractValue::Bitvector(mck::abstr::RBitvector::new(constant, *width))
            }
        }
    }
}

impl Debug for IMckNew {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bitvector(width, constant) => {
                write!(f, "::mck::Bitvector::<{}>::new({})", width, constant)
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IPhiMaybeTaken {
    pub taken: IVarId,
    pub condition: IVarId,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum IExprCall {
    //Call(WCall),
    MckUnary(IMckUnary),
    MckBinary(IMckBinary),
    //MckExt(IMckExt),
    MckNew(IMckNew),
    /*StdClone(IVarId),
    ArrayRead(IArrayRead),
    ArrayWrite(IArrayWrite),*/
    Phi(IVarId, IVarId),
    PhiTaken(IVarId),
    PhiMaybeTaken(IPhiMaybeTaken),
    PhiNotTaken,
    PhiUninit,
}

impl IExprCall {
    pub fn forward_interpret(&self, abstr: &mut Interpretation<IAbstractValue>) -> IAbstractValue {
        match self {
            IExprCall::MckUnary(unary) => unary.forward_interpret(abstr),
            IExprCall::MckBinary(binary) => binary.forward_interpret(abstr),
            IExprCall::MckNew(mck_new) => mck_new.forward_interpret(),
            IExprCall::Phi(left, right) => {
                // join the left and right variable
                let left = abstr.value(*left);
                let right = abstr.value(*right);

                left.join(right)
            }
            IExprCall::PhiTaken(taken) => abstr.value(*taken).clone(),
            IExprCall::PhiMaybeTaken(maybe_taken) => {
                // take the value normally for forward intepretation
                abstr.value(maybe_taken.taken).clone()
            }
            IExprCall::PhiNotTaken => IAbstractValue::Absent,
            IExprCall::PhiUninit => panic!("Phi uninit should not be in interpretation"),
        }
    }
    pub fn backward_interpret(
        &self,
        abstr: &Interpretation<IAbstractValue>,
        refin: &mut Interpretation<IRefinementValue>,
        later: IRefinementValue,
    ) {
        match self {
            IExprCall::MckUnary(unary) => unary.backward_interpret(abstr, refin, later),
            IExprCall::MckBinary(binary) => binary.backward_interpret(abstr, refin, later),
            IExprCall::MckNew(_) => {
                // there is no variable to propagate to, do nothing
            }
            _ => todo!(),
        }
    }
}

impl Debug for IExprCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IExprCall::MckUnary(unary) => unary.fmt(f),
            IExprCall::MckBinary(binary) => binary.fmt(f),
            IExprCall::MckNew(mck_new) => mck_new.fmt(f),
            IExprCall::Phi(left, right) => write!(f, "Phi({:?},{:?})", left, right),
            IExprCall::PhiTaken(taken) => write!(f, "PhiTaken({:?})", taken),
            IExprCall::PhiMaybeTaken(maybe_taken) => write!(
                f,
                "PhiMaybeTaken({:?},{:?})",
                maybe_taken.taken, maybe_taken.condition
            ),
            IExprCall::PhiNotTaken => write!(f, "PhiNotTaken()"),
            IExprCall::PhiUninit => write!(f, "PhiUninit()"),
        }
    }
}
