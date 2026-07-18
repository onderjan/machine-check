use indexmap::IndexMap;
use machine_check_common::{
    iir::{
        expr::{IExpr, IExprField, IExprReference, IExprStruct},
        path::{IIdent, ISpan},
        ty::{IElementaryType, IGeneralType, IType},
        variable::IVarId,
    },
    ir_common::IrReference,
};

use crate::{
    into_iir::{error, func::WFnData},
    wir::{WExpr, WExprField, WExprHighCall, WExprReference, WExprStruct, WIdent, WSpan, WSpanned},
    Error,
};

impl WExpr<WExprHighCall> {
    pub(super) fn into_iir(self, fn_data: &WFnData) -> Result<Option<IExpr>, Error> {
        Ok(Some(match self {
            WExpr::Move(ident) => {
                let var_id = from_variable_map(ident, fn_data)?;
                IExpr::Move(var_id)
            }
            WExpr::Call(expr_call) => todo!("Convert call into IIR: {:?}", expr_call),
            /* IExpr::Call(match expr_call {
                WExprHighCall::Call(call) => {
                    let unresolved_fn = || {
                        Err(error(
                            String::from("Unresolved function call"),
                            call.fn_path.wir_span(),
                        ))
                    };

                    // pop last segment and hopefully get the struct
                    let mut call_path = call.fn_path.clone();
                    let Some(call_ident) = call_path.segments.pop() else {
                        return unresolved_fn();
                    };
                    let call_ident = call_ident.ident.into_iir();

                    let Some(struct_ident) = call_path.get_ident() else {
                        return unresolved_fn();
                    };

                    let Some((struct_index, struct_data)) =
                        fn_data.struct_index_and_data(&struct_ident.clone().into_iir())
                    else {
                        return unresolved_fn();
                    };

                    let Some((fn_index, _, call_declaration)) =
                        struct_data.fns.get_full(&(ITrait::Inherent, call_ident))
                    else {
                        return unresolved_fn();
                    };

                    assert_eq!(call_declaration.signature.inputs.len(), call.args.len());

                    let mut args = Vec::new();

                    for arg in call.args {
                        match arg {
                            WCallArg::Ident(ident) => {
                                let arg = from_variable_map(ident, fn_data)?;
                                args.push(arg);
                            }
                            WCallArg::Literal(lit) => {
                                return Err(error(
                                    String::from("Non-literal argument expected"),
                                    WSpan::from_syn(&lit),
                                ));
                            }
                        }
                    }

                    IExprCall::Call(ICall {
                        func: IFnId {
                            struct_id: IStructId(struct_index),
                            fn_index,
                        },
                        args,
                    })
                }
                WExprHighCall::MckUnary(mck_unary) => {
                    let operand = from_variable_map(mck_unary.operand, fn_data)?;
                    IExprCall::MckUnary(IMckUnary {
                        op: mck_unary.op,
                        operand,
                    })
                }
                WExprCall::MckBinary(mck_binary) => {
                    let a = from_variable_map(mck_binary.a, fn_data)?;
                    let b = from_variable_map(mck_binary.b, fn_data)?;
                    IExprCall::MckBinary(IMckBinary {
                        op: mck_binary.op,
                        a,
                        b,
                    })
                }
                WExprCall::MckExt(mck_ext) => {
                    let inner = from_variable_map(mck_ext.from, fn_data)?;

                    IExprCall::MckExt(IMckExt {
                        signed: mck_ext.signed,
                        width: mck_ext.width,
                        inner,
                    })
                }
                WExprCall::MckNew(mck_new) => IExprCall::MckNew(match mck_new {
                    WMckNew::Bitvector(bitvector) => IMckNew::Bitvector(bitvector),
                    WMckNew::BitvectorArray(type_array, element_ident) => {
                        let element = from_variable_map(element_ident, fn_data)?;
                        IMckNew::BitvectorArray(type_array, element)
                    }
                }),
                WExprCall::BooleanNew(value) => IExprCall::BooleanNew(value),
                WExprCall::StdClone(ident) => {
                    let var_id = from_variable_map(ident, fn_data)?;
                    IExprCall::StdClone(var_id)
                }
                WExprCall::ArrayRead(array_read) => IExprCall::ArrayRead(IArrayRead {
                    base: from_variable_map(array_read.base, fn_data)?,
                    index: from_variable_map(array_read.index, fn_data)?,
                }),
                WExprCall::ArrayWrite(array_write) => IExprCall::ArrayWrite(IArrayWrite {
                    base: from_variable_map(array_write.base, fn_data)?,
                    index: from_variable_map(array_write.index, fn_data)?,
                    element: from_variable_map(array_write.element, fn_data)?,
                }),

                WExprCall::Phi(phi) => {
                    let condition = from_variable_map(phi.condition, fn_data)?;
                    let left = from_variable_map(phi.then_ident, fn_data)?;
                    let right = from_variable_map(phi.else_ident, fn_data)?;
                    IExprCall::Phi(IPhi {
                        condition,
                        then_var_id: left,
                        else_var_id: right,
                    })
                }
                WExprCall::PhiTaken(taken) => {
                    // translate as a move, ignore condition
                    let var_id = from_variable_map(taken.ident, fn_data)?;
                    return Ok(Some(IExpr::Move(var_id)));
                }
                WExprCall::PhiNotTaken => {
                    // do not translate to IIR as it is not needed there
                    return Ok(None);
                }
            }),*/
            WExpr::Field(expr_field) => IExpr::Field(expr_field.into_iir(fn_data)?),
            WExpr::Struct(expr_struct) => IExpr::Struct(expr_struct.into_iir(fn_data)?),
            WExpr::Reference(expr_reference) => IExpr::Reference(match expr_reference {
                WExprReference::Ident(ident) => {
                    IExprReference::Ident(from_variable_map(ident, fn_data)?)
                }
                WExprReference::Field(expr_field) => {
                    IExprReference::Field(expr_field.into_iir(fn_data)?)
                }
            }),
            WExpr::Lit(lit, _neg) => {
                return Err(error(
                    String::from("Unexpected literal"),
                    WSpan::from_syn(&lit),
                ));
            }
        }))
    }
}

impl WExprField {
    pub(super) fn into_iir(self, fn_data: &WFnData) -> Result<IExprField, Error> {
        let base_span = self.base.wir_span();
        let base_var_id = from_variable_map(self.base, fn_data)?;
        let base_var_info = fn_data.var_data(base_var_id);

        let fields = match &base_var_info.ty {
            IGeneralType::Normal(IType { inner, .. }) => {
                let unresolved_base =
                    || Err(error(String::from("Unresolved field base"), base_span));

                let IElementaryType::Struct(base_struct_id) = inner else {
                    return unresolved_base();
                };

                let Some(base_ty) = fn_data.struct_data_by_id(*base_struct_id) else {
                    return unresolved_base();
                };
                base_ty.fields.clone()
            }
            IGeneralType::PhiArg(_) => {
                panic!(
                    "Field variable type {:?} should be not be phi arg",
                    base_var_info.ty
                )
            }
        };

        let member_span = self.member.wir_span();
        let member_ident = self.member.into_iir();

        let Some(member_index) = fields.get_index_of(&member_ident) else {
            return Err(error(String::from("Unresolved field member"), member_span));
        };

        Ok(IExprField {
            base: base_var_id,
            member_index,
        })
    }
}

impl WExprStruct {
    pub(super) fn into_iir(self, fn_data: &WFnData) -> Result<IExprStruct, Error> {
        let base_type_span = self.type_path.wir_span();
        let unresolved_struct_type = || {
            Err(error(
                String::from("Unresolved struct type"),
                base_type_span,
            ))
        };

        let base_path = self.type_path;
        let Some(base_ident) = base_path.get_ident() else {
            return unresolved_struct_type();
        };
        let base_ident = base_ident.clone().into_iir();

        let Some(base_ty) = fn_data.struct_data(&base_ident) else {
            return unresolved_struct_type();
        };

        let num_fields = base_ty.fields.len();

        let mut fields = vec![None; num_fields];

        for (field_key, field_value) in self.fields {
            let field_key_span = field_key.wir_span();
            let field_key = field_key.into_iir();
            let Some(member_index) = base_ty.fields.get_index_of(&field_key) else {
                return Err(error(String::from("Field not in struct"), field_key_span));
            };

            let field_var = from_variable_map(field_value, fn_data)?;

            fields[member_index] = Some(field_var);
        }

        let mut result = Vec::with_capacity(num_fields);

        for field in fields {
            let Some(field) = field else {
                return Err(error(String::from("Missing struct field"), base_type_span));
            };
            result.push(field);
        }

        Ok(IExprStruct { fields: result })
    }
}

fn from_variable_map(ident: WIdent, fn_data: &WFnData) -> Result<IVarId, Error> {
    let span = ident.wir_span();
    let ident = ident.into_iir();
    if let Some(local_var_id) = fn_data.ident_var(&ident) {
        Ok(local_var_id)
    } else {
        Err(error(
            String::from("Identifier not found in variables"),
            span,
        ))
    }
}
