use std::fmt::Debug;

use crate::iir::{
    expr::op::{IMckBinary, IMckUnary},
    interpretation::{IAbstractValue, IRefinementValue, Interpretation},
    variable::IVarId,
};

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
