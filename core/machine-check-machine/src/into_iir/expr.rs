use machine_check_common::iir::{
    expr::{
        call::{IArrayRead, IExprCall, IMckNew, IPhiTaken},
        op::{IMckBinary, IMckExt, IMckUnary},
        IExpr, IExprField, IExprReference, IExprStruct,
    },
    ty::{IElementaryType, IGeneralType, IType},
    variable::IVarId,
};

use crate::{
    into_iir::func::WFnData,
    wir::{WExpr, WExprCall, WExprField, WExprReference, WExprStruct, WIdent, WMckNew},
};

impl WExpr<WExprCall> {
    pub(super) fn into_iir(self, fn_data: &WFnData) -> Option<IExpr> {
        Some(match self {
            WExpr::Move(ident) => {
                let var_id = from_variable_map(ident, fn_data);
                IExpr::Move(var_id)
            }
            WExpr::Call(expr_call) => IExpr::Call(match expr_call {
                WExprCall::Call(call) => todo!(),
                WExprCall::MckUnary(mck_unary) => {
                    let operand = from_variable_map(mck_unary.operand, fn_data);
                    IExprCall::MckUnary(IMckUnary {
                        op: mck_unary.op,
                        operand,
                    })
                }
                WExprCall::MckBinary(mck_binary) => {
                    let a = from_variable_map(mck_binary.a, fn_data);
                    let b = from_variable_map(mck_binary.b, fn_data);
                    IExprCall::MckBinary(IMckBinary {
                        op: mck_binary.op,
                        a,
                        b,
                    })
                }
                WExprCall::MckExt(mck_ext) => {
                    let inner = from_variable_map(mck_ext.from, fn_data);

                    IExprCall::MckExt(IMckExt {
                        signed: mck_ext.signed,
                        width: mck_ext.width,
                        inner,
                    })
                }
                WExprCall::MckNew(mck_new) => IExprCall::MckNew(match mck_new {
                    WMckNew::Bitvector(width, constant) => IMckNew::Bitvector(width, constant),
                    WMckNew::BitvectorArray(wtype_array, wident) => todo!(),
                }),
                WExprCall::BooleanNew(value) => IExprCall::BooleanNew(value),
                WExprCall::StdClone(ident) => {
                    let var_id = from_variable_map(ident, fn_data);
                    IExprCall::StdClone(var_id)
                }
                WExprCall::ArrayRead(array_read) => IExprCall::ArrayRead(IArrayRead {
                    base: from_variable_map(array_read.base, fn_data),
                    index: from_variable_map(array_read.index, fn_data),
                }),
                WExprCall::ArrayWrite(warray_write) => todo!(),
                WExprCall::Phi(left, right) => {
                    let left = from_variable_map(left, fn_data);
                    let right = from_variable_map(right, fn_data);
                    IExprCall::Phi(left, right)
                }
                WExprCall::PhiTaken(taken) => {
                    // translate as a move
                    let var = from_variable_map(taken.ident, fn_data);
                    let condition = from_variable_map(taken.condition, fn_data);
                    IExprCall::PhiTaken(IPhiTaken { var, condition })
                }
                WExprCall::PhiNotTaken => {
                    // do not translate to IIR as it is not needed there
                    return None;
                }
                WExprCall::PhiUninit => panic!("Phi uninit should not be here"),
            }),
            WExpr::Field(expr_field) => IExpr::Field(expr_field.into_iir(fn_data)),
            WExpr::Struct(expr_struct) => IExpr::Struct(expr_struct.into_iir(fn_data)),
            WExpr::Reference(expr_reference) => IExpr::Reference(match expr_reference {
                WExprReference::Ident(ident) => {
                    IExprReference::Ident(from_variable_map(ident, fn_data))
                }
                WExprReference::Field(expr_field) => {
                    IExprReference::Field(expr_field.into_iir(fn_data))
                }
            }),
            WExpr::Lit(lit) => todo!(),
        })
    }
}

impl WExprField {
    pub(super) fn into_iir(self, fn_data: &WFnData) -> IExprField {
        let base_var_id = from_variable_map(self.base, fn_data);
        let base_var_info = fn_data
            .var_data(base_var_id)
            .expect("Base field variable should have info");
        let IGeneralType::Normal(IType { inner, .. }) = &base_var_info.ty else {
            panic!(
                "Field variable type {:?} should be normal",
                base_var_info.ty
            );
        };

        let IElementaryType::Path(base_path) = inner else {
            panic!("Field variable type {:?} should be path-based", inner);
        };

        let Some(base_ident) = base_path.get_ident() else {
            panic!("Field variable type {:?} should be ident", base_path);
        };

        let Some(base_ty) = fn_data.struct_data(base_ident) else {
            panic!(
                "Field variable type {:?} should be in struct data",
                base_ident
            );
        };

        let member_ident = self.member.into_iir();

        let Some(member_index) = base_ty.fields.get_index_of(&member_ident) else {
            panic!(
                "Struct {:?} should have a field {:?}",
                base_ident, member_ident
            );
        };

        IExprField {
            base: base_var_id,
            member_index,
        }
    }
}

impl WExprStruct {
    pub(super) fn into_iir(self, fn_data: &WFnData) -> IExprStruct {
        let base_path = self.type_path.into_iir();
        let Some(base_ident) = base_path.get_ident() else {
            panic!("Structuring type path {:?} should be ident", base_path);
        };

        let Some(base_ty) = fn_data.struct_data(base_ident) else {
            panic!("Structuring type {:?} should be in struct data", base_ident);
        };

        let num_fields = base_ty.fields.len();

        let mut fields = vec![None; num_fields];

        for (field_key, field_value) in self.fields {
            let field_key = field_key.into_iir();
            let Some(member_index) = base_ty.fields.get_index_of(&field_key) else {
                panic!(
                    "Structuring {:?} field key {:?} should be in struct",
                    base_ident, field_key
                );
            };

            let field_var = from_variable_map(field_value, fn_data);

            fields[member_index] = Some(field_var);
        }

        let mut result = Vec::with_capacity(num_fields);

        for field in fields {
            let Some(field) = field else {
                panic!("All fields should be structured");
            };
            result.push(field);
        }

        IExprStruct { fields: result }
    }
}

fn from_variable_map(ident: WIdent, fn_data: &WFnData) -> IVarId {
    let ident = ident.into_iir();
    if let Some(local_var_id) = fn_data.ident_var(&ident) {
        local_var_id
    } else {
        panic!(
            "Expression variable {:?} should be in local or global variable map",
            ident
        );
    }
}
