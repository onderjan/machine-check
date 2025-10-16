use std::fmt::Debug;

use mck::three_valued::ThreeValued;

use mck::{abstr::AbstractValue, misc::Join, refin::RefinementValue};

use crate::iir::{
    expr::op::{IMckBinary, IMckUnary},
    interpretation::Interpretation,
    variable::IVarId,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum IMckNew {
    Bitvector(u32, i128),
    // TODO: bitvector array
    //BitvectorArray(WTypeArray, WIdent),
}

impl IMckNew {
    fn forward_interpret(&self) -> AbstractValue {
        match self {
            IMckNew::Bitvector(width, constant) => {
                let Ok(constant) = u64::try_from(*constant) else {
                    panic!("Constant outside u64");
                };
                AbstractValue::Bitvector(mck::abstr::RBitvector::new(constant, *width))
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
pub struct IArrayRead {
    pub base: IVarId,
    pub index: IVarId,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum IExprCall {
    //Call(WCall),
    MckUnary(IMckUnary),
    MckBinary(IMckBinary),
    //MckExt(IMckExt),
    MckNew(IMckNew),
    BooleanNew(bool),
    //StdClone(IVarId),
    ArrayRead(IArrayRead),
    //ArrayWrite(IArrayWrite),
    Phi(IVarId, IVarId),
    PhiTaken(IPhiTaken),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IPhiTaken {
    pub var: IVarId,
    pub condition: IVarId,
}

impl IExprCall {
    pub fn forward_interpret(
        &self,
        abstr: &Interpretation<AbstractValue>,
    ) -> Option<AbstractValue> {
        Some(match self {
            IExprCall::MckUnary(unary) => unary.forward_interpret(abstr),
            IExprCall::MckBinary(binary) => binary.forward_interpret(abstr),
            IExprCall::MckNew(mck_new) => mck_new.forward_interpret(),
            IExprCall::BooleanNew(value) => AbstractValue::Boolean(
                mck::abstr::Boolean::from_three_valued(ThreeValued::from_bool(*value)),
            ),
            IExprCall::ArrayRead(array_read) => {
                let array = abstr.value(array_read.base).expect_array();

                todo!("Array read")
            }
            IExprCall::Phi(left, right) => {
                // join the left and right variable value
                // at least one must be present, but not necessarily both
                let left = abstr.value_opt(*left);
                let right = abstr.value_opt(*right);

                match (left, right) {
                    (Some(left), Some(right)) => left.clone().join(right),
                    (Some(left), None) => left.clone(),
                    (None, Some(right)) => right.clone(),
                    (None, None) => panic!("At least one phi variable should be present"),
                }
            }
            IExprCall::PhiTaken(taken) => {
                // just return the value
                abstr.value(taken.var).clone()
            }
        })
    }
    pub fn backward_interpret(
        &self,
        abstr: &Interpretation<AbstractValue>,
        refin: &mut Interpretation<RefinementValue>,
        later: RefinementValue,
    ) {
        match self {
            IExprCall::MckUnary(unary) => unary.backward_interpret(abstr, refin, later),
            IExprCall::MckBinary(binary) => binary.backward_interpret(abstr, refin, later),
            IExprCall::MckNew(_) | IExprCall::BooleanNew(_) => {
                // there is no variable to propagate to, do nothing
            }
            IExprCall::Phi(a, b) => {
                // propagate into both
                refin.insert_value(*a, later.clone());
                refin.insert_value(*b, later);
            }
            IExprCall::PhiTaken(taken) => {
                // propagate into taken
                refin.insert_value(taken.var, later.clone());

                // convert to condition and propagate
                let condition_value = RefinementValue::Boolean(match later {
                    RefinementValue::Bitvector(bitvector) => bitvector.to_condition(),
                    RefinementValue::Boolean(boolean) => boolean,
                    RefinementValue::PanicResult(_) => {
                        panic!("Panic result should never be joined")
                    }
                });

                refin.join_value(taken.condition, condition_value)
            }
            IExprCall::ArrayRead(iarray_read) => todo!(),
        }
    }
}

impl Debug for IExprCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IExprCall::MckUnary(unary) => unary.fmt(f),
            IExprCall::MckBinary(binary) => binary.fmt(f),
            IExprCall::MckNew(mck_new) => mck_new.fmt(f),
            IExprCall::ArrayRead(array_read) => {
                write!(f, "ArrayRead({:?},{:?})", array_read.base, array_read.index)
            }
            IExprCall::BooleanNew(value) => write!(f, "Boolean({:?})", value),
            IExprCall::Phi(left, right) => write!(f, "Phi({:?}, {:?})", left, right),
            IExprCall::PhiTaken(taken) => {
                write!(f, "PhiTaken({:?}, {:?})", taken.var, taken.condition)
            }
        }
    }
}
