use indexmap::IndexMap;
use machine_check_common::{
    iir::{
        description::{IFnId, IStructId, ITrait},
        expr::{
            call::{IArrayRead, IArrayWrite, ICall, IExprCall, IMckNew, IPhi},
            op::{IMckBinary, IMckExt, IMckUnary},
            IExpr, IExprField, IExprReference, IExprStruct,
        },
        path::{IIdent, ISpan},
        ty::{IElementaryType, IGeneralType, IType},
        variable::IVarId,
    },
    ir_common::IrReference,
};

use crate::{
    into_iir::func::WFnData,
    wir::{WCallArg, WExpr, WExprCall, WExprField, WExprReference, WExprStruct, WIdent, WMckNew},
};

impl WExpr<WExprCall> {
    pub(super) fn into_iir(self, fn_data: &WFnData) -> Option<IExpr> {
        Some(match self {
            WExpr::Move(ident) => {
                let var_id = from_variable_map(ident, fn_data);
                IExpr::Move(var_id)
            }
            WExpr::Call(expr_call) => IExpr::Call(match expr_call {
                WExprCall::Call(call) => {
                    // pop last segment and hopefully get the struct
                    let mut call_path = call.fn_path.clone();
                    let Some(call_ident) = call_path.segments.pop() else {
                        panic!(
                            "Unresolved call {:?} should have struct parent",
                            call.fn_path
                        );
                    };
                    let call_ident = call_ident.ident.into_iir();

                    let Some(struct_ident) = call_path.get_ident() else {
                        panic!(
                            "Unresolved call {:?} should point to a struct",
                            call.fn_path
                        );
                    };

                    let Some((struct_index, struct_data)) =
                        fn_data.struct_index_and_data(&struct_ident.clone().into_iir())
                    else {
                        panic!(
                            "No known struct parent for called function {:?}",
                            call.fn_path
                        );
                    };

                    let Some((fn_index, _, call_declaration)) =
                        struct_data.fns.get_full(&(ITrait::Inherent, call_ident))
                    else {
                        panic!("Unresolved call {:?} not found in struct", call.fn_path);
                    };

                    assert_eq!(call_declaration.signature.inputs.len(), call.args.len());

                    let mut args = Vec::new();

                    for arg in call.args {
                        let WCallArg::Ident(arg) = arg else {
                            panic!("Normal call should have ident arguments");
                        };
                        let arg = from_variable_map(arg, fn_data);
                        args.push(arg);
                    }

                    IExprCall::Call(ICall {
                        func: IFnId {
                            struct_id: IStructId(struct_index),
                            fn_index,
                        },
                        args,
                    })
                }
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
                    WMckNew::BitvectorArray(_wtype_array, _wident) => todo!(),
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
                WExprCall::ArrayWrite(array_write) => IExprCall::ArrayWrite(IArrayWrite {
                    base: from_variable_map(array_write.base, fn_data),
                    index: from_variable_map(array_write.index, fn_data),
                    element: from_variable_map(array_write.element, fn_data),
                }),

                WExprCall::Phi(phi) => {
                    let condition = from_variable_map(phi.condition, fn_data);
                    let left = from_variable_map(phi.then_ident, fn_data);
                    let right = from_variable_map(phi.else_ident, fn_data);
                    IExprCall::Phi(IPhi {
                        condition,
                        then_var_id: left,
                        else_var_id: right,
                    })
                }
                WExprCall::PhiTaken(taken) => {
                    // translate as a move, ignore condition
                    let var_id = from_variable_map(taken.ident, fn_data);
                    return Some(IExpr::Move(var_id));
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
            WExpr::Lit(_lit) => panic!("Unexpected literal"),
        })
    }
}

impl WExprField {
    pub(super) fn into_iir(self, fn_data: &WFnData) -> IExprField {
        let base_var_id = from_variable_map(self.base, fn_data);
        let base_var_info = fn_data
            .var_data(base_var_id)
            .expect("Base field variable should have info");

        let fields = match &base_var_info.ty {
            IGeneralType::Normal(IType { inner, .. }) => {
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
                base_ty.fields.clone()
            }
            IGeneralType::PanicResult(ty) => {
                assert_eq!(ty.reference, IrReference::None);

                let mut fields = IndexMap::new();

                fields.insert(
                    IIdent::new(String::from("result"), ISpan::Unspecified),
                    ty.inner.clone(),
                );

                fields.insert(
                    IIdent::new(String::from("panic"), ISpan::Unspecified),
                    IElementaryType::Bitvector(32),
                );

                fields
            }
            IGeneralType::PhiArg(_) => {
                panic!(
                    "Field variable type {:?} should be not be phi arg",
                    base_var_info.ty
                )
            }
        };

        let member_ident = self.member.into_iir();

        let Some(member_index) = fields.get_index_of(&member_ident) else {
            panic!(
                "Struct {:?} should have a field '{:?}'",
                base_var_info.ty, member_ident
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
