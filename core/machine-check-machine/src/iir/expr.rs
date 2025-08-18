use std::collections::HashMap;

use crate::{
    iir::{
        interpretation::{FBitvector, IValue, Interpretation},
        variable::IVarId,
        FromWirData,
    },
    wir::{WExpr, WExprCall, WIdent, WMckBinaryOp, WMckUnaryOp, WType, ZConverted},
};

#[derive(Clone, Debug, Hash)]
pub struct IMckUnary {
    pub op: WMckUnaryOp,
    pub operand: IVarId,
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
            crate::wir::WMckBinaryOp::BitAnd => todo!(),
            crate::wir::WMckBinaryOp::BitOr => todo!(),
            crate::wir::WMckBinaryOp::BitXor => todo!(),
            crate::wir::WMckBinaryOp::LogicShl => todo!(),
            crate::wir::WMckBinaryOp::LogicShr => todo!(),
            crate::wir::WMckBinaryOp::ArithShr => todo!(),
            crate::wir::WMckBinaryOp::Add => todo!(),
            crate::wir::WMckBinaryOp::Sub => todo!(),
            crate::wir::WMckBinaryOp::Mul => todo!(),
            crate::wir::WMckBinaryOp::Udiv => todo!(),
            crate::wir::WMckBinaryOp::Urem => todo!(),
            crate::wir::WMckBinaryOp::Sdiv => todo!(),
            crate::wir::WMckBinaryOp::Srem => todo!(),
            crate::wir::WMckBinaryOp::Eq => {
                assert_eq!(a.width, b.width);
                let result = mck::forward::TypedEq::eq(a.inner, b.inner);
                IValue::Bool(result)
            }
            crate::wir::WMckBinaryOp::Ne => todo!(),
            crate::wir::WMckBinaryOp::Ult => todo!(),
            crate::wir::WMckBinaryOp::Ule => todo!(),
            crate::wir::WMckBinaryOp::Slt => todo!(),
            crate::wir::WMckBinaryOp::Sle => todo!(),
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
                IValue::Bitvector(FBitvector {
                    width: *width,
                    inner: mck::abstr::Bitvector::new(constant),
                })
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
            IExprCall::MckUnary(unary) => todo!(),
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
                    crate::wir::WMckNew::Bitvector(width, constant) => {
                        IMckNew::Bitvector(width, constant)
                    }
                    crate::wir::WMckNew::BitvectorArray(wtype_array, wident) => todo!(),
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
