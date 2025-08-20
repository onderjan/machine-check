use crate::{
    iir::{
        interpretation::{IAbstractValue, IRefinementValue, Interpretation},
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

    fn backward_interpret(&self, inter: &mut Interpretation, later: IRefinementValue) {
        let operand = inter.abstract_value(self.operand).expect_bitvector();
        let earlier = match self.op {
            WMckUnaryOp::Not => IRefinementValue::Bitvector(
                mck::backward::Bitwise::bit_not((operand,), later.expect_bitvector()).0,
            ),
            WMckUnaryOp::Neg => IRefinementValue::Bitvector(
                mck::backward::HwArith::arith_neg((operand,), later.expect_bitvector()).0,
            ),
        };

        inter.insert_refinement_value(self.operand, earlier);
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

    fn backward_interpret(&self, inter: &mut Interpretation, later: IRefinementValue) {
        let a = inter.compute_abstract_value(self.a);
        let b = inter.compute_abstract_value(self.b);

        fn handle_standard(
            a: IAbstractValue,
            b: IAbstractValue,
            later: IRefinementValue,
            func: fn(
                (mck::abstr::RBitvector, mck::abstr::RBitvector),
                mck::refin::RBitvector,
            ) -> (mck::refin::RBitvector, mck::refin::RBitvector),
        ) -> (IRefinementValue, IRefinementValue) {
            let (earlier_a, earlier_b) = (func)(
                (a.expect_bitvector(), b.expect_bitvector()),
                later.expect_bitvector(),
            );
            (
                IRefinementValue::Bitvector(earlier_a),
                IRefinementValue::Bitvector(earlier_b),
            )
        }

        fn handle_comparison(
            a: IAbstractValue,
            b: IAbstractValue,
            later: IRefinementValue,
            func: fn(
                (mck::abstr::RBitvector, mck::abstr::RBitvector),
                mck::refin::Boolean,
            ) -> (mck::refin::RBitvector, mck::refin::RBitvector),
        ) -> (IRefinementValue, IRefinementValue) {
            let (earlier_a, earlier_b) = (func)(
                (a.expect_bitvector(), b.expect_bitvector()),
                later.expect_boolean(),
            );
            (
                IRefinementValue::Bitvector(earlier_a),
                IRefinementValue::Bitvector(earlier_b),
            )
        }

        let (earlier_a, earlier_b) = match self.op {
            WMckBinaryOp::BitAnd => handle_standard(a, b, later, mck::backward::Bitwise::bit_and),
            WMckBinaryOp::BitOr => handle_standard(a, b, later, mck::backward::Bitwise::bit_or),
            WMckBinaryOp::BitXor => handle_standard(a, b, later, mck::backward::Bitwise::bit_xor),
            WMckBinaryOp::LogicShl => {
                handle_standard(a, b, later, mck::backward::HwShift::logic_shl)
            }
            WMckBinaryOp::LogicShr => {
                handle_standard(a, b, later, mck::backward::HwShift::logic_shr)
            }
            WMckBinaryOp::ArithShr => {
                handle_standard(a, b, later, mck::backward::HwShift::arith_shr)
            }
            WMckBinaryOp::Add => handle_standard(a, b, later, mck::backward::HwArith::add),
            WMckBinaryOp::Sub => handle_standard(a, b, later, mck::backward::HwArith::sub),
            WMckBinaryOp::Mul => handle_standard(a, b, later, mck::backward::HwArith::mul),
            WMckBinaryOp::Udiv => {
                todo!();
                // IAbstractValue::PanicResult(a, b, later, mck::backward::HwArith::udiv)
            }
            WMckBinaryOp::Urem => {
                todo!();
                //IAbstractValue::PanicResult(a, b, later, mck::backward::HwArith::urem)
            }
            WMckBinaryOp::Sdiv => {
                todo!();
                //IAbstractValue::PanicResult(a, b, later, mck::backward::HwArith::sdiv)
            }
            WMckBinaryOp::Srem => {
                todo!();
                //IAbstractValue::PanicResult(a, b, later, mck::backward::HwArith::srem)
            }

            WMckBinaryOp::Eq => handle_comparison(a, b, later, mck::backward::TypedEq::eq),
            WMckBinaryOp::Ne => handle_comparison(a, b, later, mck::backward::TypedEq::ne),
            WMckBinaryOp::Ult => handle_comparison(a, b, later, mck::backward::TypedCmp::ult),
            WMckBinaryOp::Ule => handle_comparison(a, b, later, mck::backward::TypedCmp::ule),
            WMckBinaryOp::Slt => handle_comparison(a, b, later, mck::backward::TypedCmp::slt),
            WMckBinaryOp::Sle => handle_comparison(a, b, later, mck::backward::TypedCmp::sle),

            _ => panic!("Not yet implemented: {:?}", self.op),
        };

        inter.insert_refinement_value(self.a, earlier_a);
        inter.insert_refinement_value(self.b, earlier_b);
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
    pub fn forward_interpret(&self, inter: &mut Interpretation) -> IAbstractValue {
        match self {
            IExprCall::MckUnary(unary) => unary.forward_interpret(inter),
            IExprCall::MckBinary(binary) => binary.forward_interpret(inter),
            IExprCall::MckNew(mck_new) => mck_new.forward_interpret(),
        }
    }
    pub fn backward_interpret(&self, inter: &mut Interpretation, later: IRefinementValue) {
        match self {
            IExprCall::MckUnary(unary) => unary.backward_interpret(inter, later),
            IExprCall::MckBinary(binary) => binary.backward_interpret(inter, later),
            IExprCall::MckNew(_) => {
                // there is no variable to propagate to, do nothing
            }
        }
    }
}
