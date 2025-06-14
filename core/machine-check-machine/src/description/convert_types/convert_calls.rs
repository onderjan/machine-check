use std::collections::BTreeMap;

use crate::{
    description::{Error, ErrorType},
    wir::{
        WBasicType, WCall, WExpr, WExprCall, WExprHighCall, WGeneralType, WHighMckExt, WHighMckNew,
        WIdent, WMckBinary, WMckBinaryOp, WMckExt, WMckNew, WMckUnary, WMckUnaryOp, WSpanned,
        WStdBinary, WStdBinaryOp, WStdUnary, WStdUnaryOp,
    },
};

use super::convert_basic_path;

pub fn convert_call_fn_path(
    call: WExprHighCall,
    local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
) -> Result<WExpr<WExprCall>, Error> {
    Ok(WExpr::Call(match call {
        WExprHighCall::Call(call) => WExprCall::Call(convert_call(call)),
        WExprHighCall::StdUnary(call) => WExprCall::MckUnary(convert_unary(call)),
        WExprHighCall::StdBinary(call) => WExprCall::MckBinary(convert_binary(call, local_types)?),
        WExprHighCall::MckExt(call) => WExprCall::MckExt(convert_ext(call, local_types)?),
        WExprHighCall::MckNew(call) => WExprCall::MckNew(convert_mck_new(call)),
        WExprHighCall::StdInto(call) => return Ok(WExpr::Move(call.from)),
        WExprHighCall::StdClone(ident) => WExprCall::StdClone(ident),
        WExprHighCall::ArrayRead(read) => WExprCall::ArrayRead(read),
        WExprHighCall::ArrayWrite(write) => WExprCall::ArrayWrite(write),
        WExprHighCall::Phi(a, b) => WExprCall::Phi(a, b),
        WExprHighCall::PhiTaken(ident) => WExprCall::PhiTaken(ident),
        WExprHighCall::PhiNotTaken => WExprCall::PhiNotTaken,
        WExprHighCall::PhiUninit => WExprCall::PhiUninit,
    }))
}

fn convert_call(call: WCall) -> WCall {
    let fn_path = convert_basic_path(call.fn_path);
    WCall {
        fn_path,
        args: call.args,
    }
}

fn convert_unary(call: WStdUnary) -> WMckUnary {
    let op = match call.op {
        WStdUnaryOp::Not => WMckUnaryOp::Not,
        WStdUnaryOp::Neg => WMckUnaryOp::Neg,
    };
    WMckUnary {
        op,
        operand: call.operand,
    }
}

fn convert_binary(
    call: WStdBinary,
    local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
) -> Result<WMckBinary, Error> {
    let mut left_arg = call.a;
    let mut right_arg = call.b;

    let op = match call.op {
        WStdBinaryOp::BitAnd => WMckBinaryOp::BitAnd,
        WStdBinaryOp::BitOr => WMckBinaryOp::BitOr,
        WStdBinaryOp::BitXor => WMckBinaryOp::BitXor,
        WStdBinaryOp::Shl => WMckBinaryOp::LogicShl,
        WStdBinaryOp::Shr => match signedness(&left_arg, local_types) {
            Some(true) => WMckBinaryOp::ArithShr,
            Some(false) => WMckBinaryOp::LogicShr,
            None => {
                return Err(Error::new(
                    ErrorType::CallConversionError("Cannot determine right shift signedness"),
                    left_arg.wir_span(),
                ))
            }
        },
        WStdBinaryOp::Add => WMckBinaryOp::Add,
        WStdBinaryOp::Sub => WMckBinaryOp::Sub,
        WStdBinaryOp::Mul => WMckBinaryOp::Mul,
        WStdBinaryOp::Eq => WMckBinaryOp::Eq,
        WStdBinaryOp::Ne => WMckBinaryOp::Ne,
        WStdBinaryOp::Lt | WStdBinaryOp::Le | WStdBinaryOp::Gt | WStdBinaryOp::Ge => {
            if matches!(call.op, WStdBinaryOp::Gt | WStdBinaryOp::Ge) {
                // swap arguments
                std::mem::swap(&mut left_arg, &mut right_arg);
            }

            let includes_equality = matches!(call.op, WStdBinaryOp::Le | WStdBinaryOp::Ge);

            let (Some(left_is_signed), Some(right_is_signed)) = (
                signedness(&left_arg, local_types),
                signedness(&right_arg, local_types),
            ) else {
                return Err(Error::new(
                    ErrorType::CallConversionError("Cannot determine comparison signedness"),
                    left_arg.wir_span(),
                ));
            };
            if left_is_signed != right_is_signed {
                return Err(Error::new(
                    ErrorType::CallConversionError("Signedness of compared types does not match"),
                    left_arg.wir_span(),
                ));
            }

            if left_is_signed {
                if includes_equality {
                    WMckBinaryOp::Sle
                } else {
                    WMckBinaryOp::Slt
                }
            } else if includes_equality {
                WMckBinaryOp::Ule
            } else {
                WMckBinaryOp::Ult
            }
        }
        WStdBinaryOp::Div => match signedness(&left_arg, local_types) {
            Some(true) => WMckBinaryOp::Sdiv,
            Some(false) => WMckBinaryOp::Udiv,
            None => {
                return Err(Error::new(
                    ErrorType::CallConversionError("Cannot determine division signedness"),
                    left_arg.wir_span(),
                ))
            }
        },
        WStdBinaryOp::Rem => match signedness(&left_arg, local_types) {
            Some(true) => WMckBinaryOp::Srem,
            Some(false) => WMckBinaryOp::Urem,
            None => {
                return Err(Error::new(
                    ErrorType::CallConversionError("Cannot determine remainder signedness"),
                    left_arg.wir_span(),
                ))
            }
        },
    };

    Ok(WMckBinary {
        op,
        a: left_arg,
        b: right_arg,
    })
}

fn convert_ext(
    call: WHighMckExt,
    local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
) -> Result<WMckExt, Error> {
    let Some(signed) = signedness(&call.from, local_types) else {
        return Err(Error::new(
            ErrorType::CallConversionError("Cannot determine bit extension signedness"),
            call.from.wir_span(),
        ));
    };

    Ok(WMckExt {
        signed,
        width: call.width,
        from: call.from,
    })
}

fn convert_mck_new(call: WHighMckNew) -> WMckNew {
    match call {
        WHighMckNew::Bitvector(width, constant) => WMckNew::Bitvector(width, constant),
        WHighMckNew::BitvectorArray(type_array, fill_element) => {
            WMckNew::BitvectorArray(type_array, fill_element)
        }
        WHighMckNew::Unsigned(width, constant) => WMckNew::Bitvector(width, constant),
        WHighMckNew::Signed(width, constant) => WMckNew::Bitvector(width, constant),
    }
}

fn signedness(
    ident: &WIdent,
    local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
) -> Option<bool> {
    let ty = local_types.get(ident);
    let Some(ty) = ty else {
        // type is not in local ident types, do not determine signedness
        return None;
    };
    match ty {
        WGeneralType::Normal(ty) => match ty.inner {
            WBasicType::Unsigned(_) => Some(false),
            WBasicType::Signed(_) => Some(true),
            _ => None,
        },
        _ => None,
    }
}
