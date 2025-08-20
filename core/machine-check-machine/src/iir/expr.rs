use std::collections::HashMap;

use crate::{
    iir::{
        interpretation::{IValue, Interpretation},
        variable::IVarId,
        FromWirData,
    },
    wir::{WExpr, WExprCall, WIdent, WMckBinaryOp, WMckNew, WMckUnaryOp},
};

#[derive(Clone, Debug, Hash)]
pub struct IMckUnary {
    pub op: WMckUnaryOp,
    pub operand: IVarId,
}

impl IMckUnary {
    fn interpret(&self, inter: &mut Interpretation) -> IValue {
        let operand = inter.value(self.operand).expect_bitvector();
        match self.op {
            WMckUnaryOp::Not => IValue::Bitvector(mck::forward::Bitwise::bit_not(operand)),
            WMckUnaryOp::Neg => IValue::Bitvector(mck::forward::HwArith::arith_neg(operand)),
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
    fn interpret(&self, inter: &mut Interpretation) -> IValue {
        let a = inter.value(self.a).expect_bitvector();
        let b = inter.value(self.b).expect_bitvector();

        match self.op {
            WMckBinaryOp::BitAnd => IValue::Bitvector(mck::forward::Bitwise::bit_and(a, b)),
            WMckBinaryOp::BitOr => IValue::Bitvector(mck::forward::Bitwise::bit_or(a, b)),
            WMckBinaryOp::BitXor => IValue::Bitvector(mck::forward::Bitwise::bit_xor(a, b)),
            WMckBinaryOp::LogicShl => IValue::Bitvector(mck::forward::HwShift::logic_shl(a, b)),
            WMckBinaryOp::LogicShr => IValue::Bitvector(mck::forward::HwShift::logic_shr(a, b)),
            WMckBinaryOp::ArithShr => IValue::Bitvector(mck::forward::HwShift::arith_shr(a, b)),
            WMckBinaryOp::Add => IValue::Bitvector(mck::forward::HwArith::add(a, b)),
            WMckBinaryOp::Sub => IValue::Bitvector(mck::forward::HwArith::sub(a, b)),
            WMckBinaryOp::Mul => IValue::Bitvector(mck::forward::HwArith::mul(a, b)),
            WMckBinaryOp::Udiv => IValue::PanicResult(mck::forward::HwArith::udiv(a, b)),
            WMckBinaryOp::Urem => IValue::PanicResult(mck::forward::HwArith::urem(a, b)),
            WMckBinaryOp::Sdiv => IValue::PanicResult(mck::forward::HwArith::sdiv(a, b)),
            WMckBinaryOp::Srem => IValue::PanicResult(mck::forward::HwArith::srem(a, b)),
            WMckBinaryOp::Eq => IValue::Bool(mck::forward::TypedEq::eq(a, b)),
            WMckBinaryOp::Ne => IValue::Bool(mck::forward::TypedEq::ne(a, b)),
            WMckBinaryOp::Ult => IValue::Bool(mck::forward::TypedCmp::ult(a, b)),
            WMckBinaryOp::Ule => IValue::Bool(mck::forward::TypedCmp::ule(a, b)),
            WMckBinaryOp::Slt => IValue::Bool(mck::forward::TypedCmp::slt(a, b)),
            WMckBinaryOp::Sle => IValue::Bool(mck::forward::TypedCmp::sle(a, b)),
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
    fn interpret(&self, inter: &mut Interpretation) -> IValue {
        match self {
            IMckNew::Bitvector(width, constant) => {
                let Ok(constant) = u64::try_from(*constant) else {
                    panic!("Constant outside u64");
                };
                IValue::Bitvector(mck::abstr::RBitvector::new(constant, *width))
            } //IMckNew::BitvectorArray(wtype_array, wident) => todo!(),
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
    fn interpret(&self, inter: &mut Interpretation) -> IValue {
        println!("Executing call");
        match self {
            //IExprCall::Call(wcall) => todo!(),
            IExprCall::MckUnary(unary) => unary.interpret(inter),
            IExprCall::MckBinary(binary) => binary.interpret(inter),
            //IExprCall::MckExt(wmck_ext) => todo!(),
            IExprCall::MckNew(mck_new) => mck_new.interpret(inter),
            /*IExprCall::StdClone(wident) => todo!(),
            IExprCall::ArrayRead(warray_read) => todo!(),
            IExprCall::ArrayWrite(warray_write) => todo!(),
            IExprCall::Phi(wident, wident1) => todo!(),
            IExprCall::PhiTaken(wident) => todo!(),
            IExprCall::PhiMaybeTaken(wphi_maybe_taken) => todo!(),
            IExprCall::PhiNotTaken => todo!(),
            IExprCall::PhiUninit => todo!(),*/
        }
    }
}

#[derive(Clone, Debug, Hash)]
pub enum IExpr {
    Move(IVarId),
    Call(IExprCall),
    /*Field(IExprField),
    Struct(IExprStruct),
    Reference(IExprReference),
    Lit(Lit),*/
}

impl IExpr {
    pub fn from_wir(
        data: &mut FromWirData,
        expr: WExpr<WExprCall>,
        ident_var_map: &HashMap<WIdent, IVarId>,
    ) -> Self {
        match expr {
            WExpr::Move(ident) => {
                let var_id = *ident_var_map
                    .get(&ident)
                    .expect("Left-side variable should be in variable map");
                IExpr::Move(var_id)
            }
            WExpr::Call(expr_call) => IExpr::Call(match expr_call {
                WExprCall::Call(wcall) => todo!(),
                WExprCall::MckUnary(wmck_unary) => todo!(),
                WExprCall::MckBinary(mck_binary) => {
                    let a = from_variable_map(data, &mck_binary.a, ident_var_map);
                    let b = from_variable_map(data, &mck_binary.b, ident_var_map);
                    IExprCall::MckBinary(IMckBinary {
                        op: mck_binary.op,
                        a,
                        b,
                    })
                }
                WExprCall::MckExt(wmck_ext) => todo!(),
                WExprCall::MckNew(mck_new) => IExprCall::MckNew(match mck_new {
                    WMckNew::Bitvector(width, constant) => IMckNew::Bitvector(width, constant),
                    WMckNew::BitvectorArray(wtype_array, wident) => todo!(),
                }),
                WExprCall::StdClone(wident) => todo!(),
                WExprCall::ArrayRead(warray_read) => todo!(),
                WExprCall::ArrayWrite(warray_write) => todo!(),
                WExprCall::Phi(wident, wident1) => todo!(),
                WExprCall::PhiTaken(wident) => todo!(),
                WExprCall::PhiMaybeTaken(wphi_maybe_taken) => todo!(),
                WExprCall::PhiNotTaken => todo!(),
                WExprCall::PhiUninit => todo!(),
            }),
            WExpr::Field(wexpr_field) => todo!(),
            WExpr::Struct(wexpr_struct) => todo!(),
            WExpr::Reference(wexpr_reference) => todo!(),
            WExpr::Lit(lit) => todo!(),
        }
    }

    pub fn interpret(&self, inter: &mut Interpretation) -> IValue {
        match self {
            IExpr::Move(var_id) => inter.value(*var_id).clone(),
            IExpr::Call(expr_call) => expr_call.interpret(inter),
            /*IExpr::Field(expr_field) => todo!(),
            IExpr::Struct(expr_struct) => todo!(),
            IExpr::Reference(expr_reference) => todo!(),
            IExpr::Lit(lit) => todo!(),*/
        }
    }
}

fn from_variable_map(
    data: &mut FromWirData,
    ident: &WIdent,
    ident_var_map: &HashMap<WIdent, IVarId>,
) -> IVarId {
    if let Some(local_var_id) = ident_var_map.get(ident) {
        *local_var_id
    } else if let Some(global_var_id) = data.global_var_ids.get(ident) {
        data.used_globals.insert(*global_var_id);
        *global_var_id
    } else {
        panic!(
            "Expression variable {:?} should be in local or global variable map",
            ident
        );
    }
}
