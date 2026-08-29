use crate::{
    context::{
        low::{error, func::WFnData},
        WLowContext,
    },
    wir::{
        WDatatypeId, WExpr, WExprField, WExprLowCall, WExprReference, WExprStruct, WIdent, WMckNew,
        WSpan,
    },
    Error,
};

use machine_check_common::iir::{
    expr::{
        call::{IExprCall, IMckNew, IPhi},
        op::{IMckBinary, IMckExt, IMckUnary},
        IExpr, IExprField, IExprReference, IExprStruct,
    },
    ty::{IElementaryType, IGeneralType, IType},
    variable::IVarId,
};

impl WExpr<WExprLowCall> {
    pub(super) fn into_iir(
        self,
        ctx: &WLowContext,
        fn_data: &WFnData,
    ) -> Result<Option<IExpr>, Error> {
        Ok(Some(match self {
            WExpr::Move(ident) => {
                let var_id = from_variable_map(ident, fn_data)?;
                IExpr::Move(var_id)
            }
            WExpr::Call(expr_call) => match expr_call {
                WExprLowCall::Call(call) => {
                    todo!("Call into IIR: {:?}", call)
                }
                WExprLowCall::MckUnary(unary) => {
                    let operand = from_variable_map(unary.operand, fn_data)?;
                    IExpr::Call(IExprCall::MckUnary(IMckUnary {
                        op: unary.op,
                        operand,
                    }))
                }
                WExprLowCall::MckBinary(binary) => {
                    let a = from_variable_map(binary.a, fn_data)?;
                    let b = from_variable_map(binary.b, fn_data)?;

                    IExpr::Call(IExprCall::MckBinary(IMckBinary {
                        op: binary.op,
                        a,
                        b,
                    }))
                }
                WExprLowCall::MckExt(mck_ext) => {
                    let inner = from_variable_map(mck_ext.from, fn_data)?;

                    IExpr::Call(IExprCall::MckExt(IMckExt {
                        signed: mck_ext.signed,
                        width: mck_ext.width,
                        inner,
                    }))
                }
                WExprLowCall::MckNew(mck_new) => IExpr::Call(IExprCall::MckNew(match mck_new {
                    WMckNew::Bitvector(bitvector) => IMckNew::Bitvector(bitvector),
                })),
                WExprLowCall::Phi(phi) => {
                    let condition = from_variable_map(phi.condition, fn_data)?;
                    let left = from_variable_map(phi.then_ident, fn_data)?;
                    let right = from_variable_map(phi.else_ident, fn_data)?;
                    IExpr::Call(IExprCall::Phi(IPhi {
                        condition,
                        then_var_id: left,
                        else_var_id: right,
                    }))
                }
                WExprLowCall::PhiTaken(taken) => {
                    // translate as a move, ignore condition
                    let var_id = from_variable_map(taken.ident, fn_data)?;
                    return Ok(Some(IExpr::Move(var_id)));
                }
                WExprLowCall::PhiNotTaken => {
                    // do not translate to IIR as it is not needed there
                    return Ok(None);
                }
            },
            WExpr::Field(expr_field) => IExpr::Field(expr_field.into_iir(ctx, fn_data)?),
            WExpr::Struct(expr_struct) => IExpr::Struct(expr_struct.into_iir(ctx, fn_data)?),
            WExpr::Reference(expr_reference) => IExpr::Reference(match expr_reference {
                WExprReference::Ident(ident) => {
                    IExprReference::Ident(from_variable_map(ident, fn_data)?)
                }
                WExprReference::Field(expr_field) => {
                    IExprReference::Field(expr_field.into_iir(ctx, fn_data)?)
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
    pub(super) fn into_iir(
        self,
        ctx: &WLowContext,
        fn_data: &WFnData,
    ) -> Result<IExprField, Error> {
        let base_span = self.base.span();
        let base_var_id = from_variable_map(self.base, fn_data)?;
        let base_var_info = fn_data.var_data(base_var_id);

        let fields = match &base_var_info.ty {
            IGeneralType::Normal(IType { inner, .. }) => {
                let unresolved_base =
                    || Err(error(String::from("Unresolved field base"), base_span));

                let IElementaryType::Struct(base_struct_id) = inner else {
                    return unresolved_base();
                };

                let datatype_id = WDatatypeId::from_index(base_struct_id.0);

                let Some((_base_ty_path, base_ty)) = ctx.definitions().datatype_by_id(datatype_id)
                else {
                    return unresolved_base();
                };
                base_ty.def.fields.clone()
            }
            IGeneralType::PhiArg(_) => {
                panic!(
                    "Field variable type {:?} should be not be phi arg",
                    base_var_info.ty
                )
            }
        };

        let member_span = self.member.span();

        eprintln!(
            "Member ident: {:?}, type: {:?}, fields: {:?}",
            self.member, base_var_info.ty, fields
        );

        let Some(member_index) = fields.get_index_of(&self.member) else {
            return Err(error(String::from("Unresolved field member"), member_span));
        };

        Ok(IExprField {
            base: base_var_id,
            member_index,
        })
    }
}

impl WExprStruct {
    pub(super) fn into_iir(
        self,
        ctx: &WLowContext,
        fn_data: &WFnData,
    ) -> Result<IExprStruct, Error> {
        let base_ty = ctx.id_datatype(self.ty);

        let num_fields = base_ty.def.fields.len();

        let mut fields = vec![None; num_fields];

        for (field_key, field_value) in self.fields {
            let field_key_span = field_key.span();
            let Some(member_index) = base_ty.def.fields.get_index_of(&field_key) else {
                return Err(error(String::from("Field not in struct"), field_key_span));
            };

            let field_var = from_variable_map(field_value, fn_data)?;

            fields[member_index] = Some(field_var);
        }

        let mut result = Vec::with_capacity(num_fields);

        for field in fields {
            let Some(field) = field else {
                return Err(error(
                    String::from("Missing struct field"),
                    WSpan::call_site(),
                ));
            };
            result.push(field);
        }

        Ok(IExprStruct { fields: result })
    }
}

fn from_variable_map(ident: WIdent, fn_data: &WFnData) -> Result<IVarId, Error> {
    let span = ident.span();
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
