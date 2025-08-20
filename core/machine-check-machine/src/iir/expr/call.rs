use crate::{
    iir::{
        interpretation::{IAbstractValue, Interpretation},
        variable::IVarId,
    },
    wir::{WMckBinaryOp, WMckUnaryOp},
};

#[derive(Clone, Debug, Hash)]
pub struct IMckUnary {
    pub op: WMckUnaryOp,
    pub operand: IVarId,
}

impl IMckUnary {
    fn forward_interpret(&self, inter: &mut Interpretation) -> IAbstractValue {
        let operand = inter.abstract_value(self.operand).expect_bitvector();
        match self.op {
            WMckUnaryOp::Not => IAbstractValue::Bitvector(mck::forward::Bitwise::bit_not(operand)),
            WMckUnaryOp::Neg => {
                IAbstractValue::Bitvector(mck::forward::HwArith::arith_neg(operand))
            }
        }
    }
}

#[derive(Clone, Debug, Hash)]
pub struct IMckBinary {
    pub op: WMckBinaryOp,
    pub a: IVarId,
    pub b: IVarId,
}

impl IMckBinary {
    fn forward_interpret(&self, inter: &mut Interpretation) -> IAbstractValue {
        let a = inter.abstract_value(self.a).expect_bitvector();
        let b = inter.abstract_value(self.b).expect_bitvector();

        match self.op {
            WMckBinaryOp::BitAnd => IAbstractValue::Bitvector(mck::forward::Bitwise::bit_and(a, b)),
            WMckBinaryOp::BitOr => IAbstractValue::Bitvector(mck::forward::Bitwise::bit_or(a, b)),
            WMckBinaryOp::BitXor => IAbstractValue::Bitvector(mck::forward::Bitwise::bit_xor(a, b)),
            WMckBinaryOp::LogicShl => {
                IAbstractValue::Bitvector(mck::forward::HwShift::logic_shl(a, b))
            }
            WMckBinaryOp::LogicShr => {
                IAbstractValue::Bitvector(mck::forward::HwShift::logic_shr(a, b))
            }
            WMckBinaryOp::ArithShr => {
                IAbstractValue::Bitvector(mck::forward::HwShift::arith_shr(a, b))
            }
            WMckBinaryOp::Add => IAbstractValue::Bitvector(mck::forward::HwArith::add(a, b)),
            WMckBinaryOp::Sub => IAbstractValue::Bitvector(mck::forward::HwArith::sub(a, b)),
            WMckBinaryOp::Mul => IAbstractValue::Bitvector(mck::forward::HwArith::mul(a, b)),
            WMckBinaryOp::Udiv => IAbstractValue::PanicResult(mck::forward::HwArith::udiv(a, b)),
            WMckBinaryOp::Urem => IAbstractValue::PanicResult(mck::forward::HwArith::urem(a, b)),
            WMckBinaryOp::Sdiv => IAbstractValue::PanicResult(mck::forward::HwArith::sdiv(a, b)),
            WMckBinaryOp::Srem => IAbstractValue::PanicResult(mck::forward::HwArith::srem(a, b)),
            WMckBinaryOp::Eq => IAbstractValue::Bool(mck::forward::TypedEq::eq(a, b)),
            WMckBinaryOp::Ne => IAbstractValue::Bool(mck::forward::TypedEq::ne(a, b)),
            WMckBinaryOp::Ult => IAbstractValue::Bool(mck::forward::TypedCmp::ult(a, b)),
            WMckBinaryOp::Ule => IAbstractValue::Bool(mck::forward::TypedCmp::ule(a, b)),
            WMckBinaryOp::Slt => IAbstractValue::Bool(mck::forward::TypedCmp::slt(a, b)),
            WMckBinaryOp::Sle => IAbstractValue::Bool(mck::forward::TypedCmp::sle(a, b)),
        }
    }
}

#[derive(Clone, Debug, Hash)]
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

#[derive(Clone, Debug, Hash)]
pub enum IExprCall {
    //Call(WCall),
    MckUnary(IMckUnary),
    MckBinary(IMckBinary),
    //MckExt(IMckExt),
    MckNew(IMckNew),
    /*StdClone(IVarId),
    ArrayRead(IArrayRead),
    ArrayWrite(IArrayWrite),
    Phi(IVarId, IVarId),
    PhiTaken(IVarId),
    PhiMaybeTaken(IPhiMaybeTaken),
    PhiNotTaken,
    PhiUninit,*/
}

impl IExprCall {
    pub fn interpret(&self, inter: &mut Interpretation) -> IAbstractValue {
        println!("Executing call");
        match self {
            IExprCall::MckUnary(unary) => unary.forward_interpret(inter),
            IExprCall::MckBinary(binary) => binary.forward_interpret(inter),
            IExprCall::MckNew(mck_new) => mck_new.forward_interpret(),
        }
    }
}
