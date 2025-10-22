use std::fmt::Debug;

use mck::three_valued::ThreeValued;

use mck::{
    abstr::AbstractValue,
    forward::ReadWrite,
    misc::Join,
    refin::{Limit, RefinementValue},
};
use serde::{Deserialize, Serialize};

use crate::iir::context::IFnContext;
use crate::iir::description::IFnId;
use crate::iir::expr::op::IMckExt;
use crate::iir::{
    expr::op::{IMckBinary, IMckUnary},
    variable::IVarId,
};
use crate::iir::{join_limited, IAbstr, IRefin};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IArrayRead {
    pub base: IVarId,
    pub index: IVarId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IArrayWrite {
    pub base: IVarId,
    pub index: IVarId,
    pub element: IVarId,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IExprCall {
    Call(ICall),
    MckUnary(IMckUnary),
    MckBinary(IMckBinary),
    MckExt(IMckExt),
    MckNew(IMckNew),
    BooleanNew(bool),
    StdClone(IVarId),
    ArrayRead(IArrayRead),
    ArrayWrite(IArrayWrite),
    Phi(IPhi),
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IPhi {
    pub condition: IVarId,
    pub then_var_id: IVarId,
    pub else_var_id: IVarId,
}

impl IExprCall {
    pub fn forward_interpret(&self, context: &IFnContext, abstr: &IAbstr) -> Option<AbstractValue> {
        Some(match self {
            IExprCall::Call(call) => call.forward_interpret(context, abstr),
            IExprCall::MckUnary(unary) => unary.forward_interpret(abstr),
            IExprCall::MckBinary(binary) => binary.forward_interpret(abstr),
            IExprCall::MckExt(ext) => ext.forward_interpret(abstr),
            IExprCall::MckNew(mck_new) => mck_new.forward_interpret(),
            IExprCall::BooleanNew(value) => AbstractValue::Boolean(
                mck::abstr::Boolean::from_three_valued(ThreeValued::from_bool(*value)),
            ),
            IExprCall::ArrayRead(array_read) => {
                let array = abstr.value(array_read.base).expect_array();
                let index = abstr.value(array_read.index).expect_bitvector();

                AbstractValue::Bitvector(array.read(*index))
            }
            IExprCall::ArrayWrite(array_write) => {
                let array = abstr.value(array_write.base).expect_array();
                let index = abstr.value(array_write.index).expect_bitvector();
                let element = abstr.value(array_write.element).expect_bitvector();

                AbstractValue::Array(array.write(*index, *element))
            }
            IExprCall::Phi(phi) => {
                // join the left and right variable value
                // at least one must be present, but not necessarily both
                let left = abstr.value_opt(phi.then_var_id);
                let right = abstr.value_opt(phi.else_var_id);

                match (left, right) {
                    (Some(left), Some(right)) => left.clone().join(right),
                    (Some(left), None) => left.clone(),
                    (None, Some(right)) => right.clone(),
                    (None, None) => panic!("At least one phi variable should be present"),
                }
            }
            IExprCall::StdClone(var_id) => {
                // clone
                abstr.value(*var_id).clone()
            }
        })
    }
    pub fn backward_interpret(
        &self,
        context: &IFnContext,
        abstr: &IAbstr,
        refin: &mut IRefin,
        refin_later: RefinementValue,
    ) {
        match self {
            IExprCall::Call(call) => call.backward_interpret(context, abstr, refin, refin_later),
            IExprCall::MckUnary(unary) => unary.backward_interpret(abstr, refin, refin_later),
            IExprCall::MckBinary(binary) => binary.backward_interpret(abstr, refin, refin_later),
            IExprCall::MckExt(ext) => ext.backward_interpret(abstr, refin, refin_later),
            IExprCall::MckNew(_) | IExprCall::BooleanNew(_) => {
                // there is no variable to propagate to, do nothing
            }
            IExprCall::StdClone(var_id) => {
                // limit and join previous

                join_limited(abstr, refin, *var_id, refin_later);
            }
            IExprCall::Phi(phi) => {
                // propagate into both
                // the abstract value might not be present, limit manually, skipping when not present
                if let Some(abstr_a) = abstr.value_opt(phi.then_var_id) {
                    let refin_a = refin_later.clone().limit(abstr_a);
                    refin.join_value(phi.then_var_id, refin_a);
                }
                if let Some(abstr_b) = abstr.value_opt(phi.else_var_id) {
                    let refin_b = refin_later.clone().limit(abstr_b);
                    refin.join_value(phi.else_var_id, refin_b);
                }

                // convert to condition and propagate
                let condition_value = refin_later.to_condition();

                join_limited(
                    abstr,
                    refin,
                    phi.condition,
                    RefinementValue::Boolean(condition_value),
                )
            }
            IExprCall::ArrayRead(array_read) => {
                let refin_element = refin_later.expect_bitvector();
                let abstr_earlier = abstr.value(array_read.base).expect_array();
                let abstr_index = abstr.value(array_read.index).expect_bitvector();
                let (refin_earlier, refin_index) =
                    mck::backward::ReadWrite::read((abstr_earlier, *abstr_index), *refin_element);

                // we already have the abstract values, limit them here
                let refin_earlier = refin_earlier.limit(abstr_earlier);
                let refin_index = refin_index.limit(abstr_index);

                refin.join_value(array_read.base, RefinementValue::Array(refin_earlier));
                refin.join_value(array_read.index, RefinementValue::Bitvector(refin_index));
            }
            IExprCall::ArrayWrite(array_write) => {
                let RefinementValue::Array(refin_later) = refin_later else {
                    panic!("Array write later should be an array");
                };

                let abstr_earlier = abstr.value(array_write.base).expect_array();
                let abstr_index = abstr.value(array_write.index).expect_bitvector();
                let abstr_element = abstr.value(array_write.element).expect_bitvector();

                let (refin_earlier, refin_index, refin_element) = mck::backward::ReadWrite::write(
                    (abstr_earlier, *abstr_index, *abstr_element),
                    refin_later.clone(),
                );

                // we already have the abstract values, limit them here
                let refin_earlier = refin_earlier.limit(abstr_earlier);
                let refin_index = refin_index.limit(abstr_index);
                let refin_element = refin_element.limit(abstr_element);

                refin.join_value(array_write.base, RefinementValue::Array(refin_earlier));
                refin.join_value(array_write.index, RefinementValue::Bitvector(refin_index));
                refin.join_value(
                    array_write.element,
                    RefinementValue::Bitvector(refin_element),
                );
            }
        }
    }
}

impl ICall {
    pub fn forward_interpret(&self, context: &IFnContext, abstr: &IAbstr) -> AbstractValue {
        let func = context.context.fn_with_id(self.func);
        let mut input_values = Vec::new();

        for var_id in self.args.iter().cloned() {
            let input_value = abstr.value(var_id).clone();
            input_values.push(input_value);
        }

        let (normal, panic) = func.call(context.context, input_values);
        AbstractValue::Struct(vec![normal, panic])
    }

    pub fn backward_interpret(
        &self,
        context: &IFnContext,
        abstr: &IAbstr,
        refin: &mut IRefin,
        refin_later: RefinementValue,
    ) {
        let func = context.context.fn_with_id(self.func);

        let refin_later = refin_later.expect_struct();

        let later_normal = refin_later[0].clone();
        let later_panic = refin_later[1].clone();

        let mut input_values = Vec::new();

        for var_id in self.args.iter().cloned() {
            let input_value = abstr.value(var_id).clone();
            input_values.push(input_value);
        }

        let func_abstr = func.forward_interpret(context.context, input_values);

        let func_refin =
            func.backward_interpret(context.context, &func_abstr, later_normal, later_panic);

        let refin_inputs = func.backward_earlier(&func_abstr, &func_refin);

        for (refin_var_id, refin_input) in self.args.iter().zip(refin_inputs) {
            join_limited(abstr, refin, *refin_var_id, refin_input)
        }
    }
}

impl Debug for IExprCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IExprCall::Call(call) => call.fmt(f),
            IExprCall::MckUnary(unary) => unary.fmt(f),
            IExprCall::MckBinary(binary) => binary.fmt(f),
            IExprCall::MckExt(ext) => ext.fmt(f),
            IExprCall::MckNew(mck_new) => mck_new.fmt(f),
            IExprCall::StdClone(var_id) => {
                write!(f, "StdClone({:?})", var_id)
            }
            IExprCall::ArrayRead(array_read) => {
                write!(f, "{:?}[{:?}]", array_read.base, array_read.index)
            }
            IExprCall::ArrayWrite(array_write) => {
                write!(
                    f,
                    "({:?}[{:?}] <-- {:?})",
                    array_write.base, array_write.index, array_write.element
                )
            }
            IExprCall::BooleanNew(value) => write!(f, "Boolean({:?})", value),
            IExprCall::Phi(phi) => {
                write!(
                    f,
                    "{:?} ? {:?} : {:?}",
                    phi.condition, phi.then_var_id, phi.else_var_id
                )
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ICall {
    pub func: IFnId,
    pub args: Vec<IVarId>,
}

impl Debug for ICall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}(", self.func)?;
        for arg in &self.args {
            write!(f, "{:?}, ", arg)?;
        }
        write!(f, ")")
    }
}
