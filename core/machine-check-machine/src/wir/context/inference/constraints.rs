use std::collections::BTreeMap;

use machine_check_common::ir_common::IrStdBinaryOp;
use syn::{Path, Type, TypePath};

use crate::{
    into_wir::{Error, ErrorType},
    wir::{
        context::{
            typedef::WContextTypeDef,
            types::{bitvector_type, bool_type},
        },
        signed_type, unsigned_type, WBlock, WExpr, WExprHighCall, WIdent, WIndexedExpr,
        WIndexedIdent, WMacroableStmt, WPartialArgument, WPartialType, WSpanned, WTypeId, ZTac,
    },
};

impl super::WInferenceContext {
    pub(super) fn add_block_constraints(
        &mut self,
        types: &BTreeMap<WIdent, WTypeId>,
        block: &WBlock<ZTac>,
        is_property: bool,
    ) -> Result<(), Error> {
        eprintln!("Adding constraints for block {:?}", block);
        for stmt in &block.stmts {
            eprintln!("Should add constraints for statement {:#?}", stmt);
            match stmt {
                WMacroableStmt::Assign(assign) => {
                    let left = match &assign.left {
                        WIndexedIdent::Indexed(wident, wident1) => {
                            todo!("Constraints for indexed left")
                        }
                        WIndexedIdent::NonIndexed(ident) => ident,
                    };
                    let right = match &assign.right {
                        WIndexedExpr::Indexed(base_expr, ident) => {
                            todo!("Constraints for indexed right")
                        }
                        WIndexedExpr::NonIndexed(expr) => expr,
                    };
                    eprintln!(
                        "Should add constraints for left {:?}, right {:?}",
                        left, right
                    );

                    let left_ty = get_type(types, &left)?;

                    match right {
                        WExpr::Move(wident) => todo!("Move"),
                        WExpr::Call(call) => {
                            self.add_call_constraint(types, left_ty, call)?;
                        }
                        WExpr::Field(expr_field) => {
                            let base_ty = get_type(types, &expr_field.base)?;
                            // TODO: this should be in fixpoint to work with inference well
                            let mut base_ty = &self.types[base_ty.0];

                            eprintln!("Field {:?}: base type {:?}", expr_field, base_ty);

                            while let WPartialType::Reference(ty) = base_ty {
                                base_ty = ty.as_ref();
                            }

                            if let WPartialType::Path(path) = base_ty {
                                eprintln!("Field {:?}: base path {:?}", expr_field, path);
                                let base_ty = Type::Path(TypePath {
                                    qself: None,
                                    path: Path::from(path.clone()),
                                });
                                let base_def = self.type_defs.get(&base_ty);
                                eprintln!("Base def: {:?}", base_def);
                                if let Some(WContextTypeDef::Struct(struct_def)) = base_def {
                                    for field in struct_def {
                                        if field.0 == expr_field.member {
                                            // TODO: reference
                                            eprintln!("Member type: {:?}", field);
                                            self.add_eq_constraint(
                                                left_ty.clone(),
                                                field.1.clone(),
                                            );
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        WExpr::Struct(expr_struct) => {
                            let struct_ty = self.type_id(&Type::Path(TypePath {
                                qself: None,
                                path: Path::from(expr_struct.type_path.clone()),
                            }))?;
                            self.add_eq_constraint(left_ty, struct_ty);
                        }
                        WExpr::Reference(wexpr_reference) => todo!("Reference"),
                        WExpr::Lit(lit, _) => {
                            if is_property {
                                // ignore
                            } else {
                                todo!("Literal")
                            }
                        }
                    }
                }
                WMacroableStmt::If(stmt_if) => {
                    eprintln!("Should add constraints for if {:#?}", stmt_if);
                    self.add_block_constraints(types, &stmt_if.then_block, is_property)?;
                    self.add_block_constraints(types, &stmt_if.else_block, is_property)?;
                }
                WMacroableStmt::PanicMacro(wstmt_panic_macro) => {
                    todo!("Constraints for panic macro")
                }
            }
        }
        Ok(())
    }

    fn add_call_constraint(
        &mut self,
        types: &BTreeMap<WIdent, WTypeId>,
        left_ty: WTypeId,
        call: &WExprHighCall,
    ) -> Result<(), Error> {
        eprintln!("Call");
        match call {
            WExprHighCall::Call(call) => {
                let mut found = false;
                if call.fn_path.leading_colon.is_some() && call.fn_path.segments.len() == 3 {
                    let segments = &call.fn_path.segments;
                    let is_bitvector = segments[1].ident.name() == "Bitvector";
                    let is_unsigned = segments[1].ident.name() == "Unsigned";
                    let is_signed = segments[1].ident.name() == "Signed";

                    eprintln!(
                        "Bitvector: {}, unsigned: {}, signed: {}",
                        is_bitvector, is_unsigned, is_signed
                    );

                    if segments[0].ident.name() == "machine_check"
                        && (is_bitvector || is_unsigned || is_signed)
                        && segments[2].ident.name() == "new"
                    {
                        let mut width = None;
                        if let Some(generics) = &segments[1].generics {
                            if generics.arguments.len() == 1 {
                                if let WPartialArgument::Uint(width_arg, _span) =
                                    generics.arguments[0]
                                {
                                    width = Some(width_arg)
                                }
                            }
                        }
                        let ty = if is_bitvector {
                            bitvector_type(width)
                        } else if is_unsigned {
                            unsigned_type(width)
                        } else {
                            signed_type(width)
                        };

                        // constrain the output to be a bitvector of the given width
                        let bitvector_ty = self.partial_type_id(ty);
                        self.add_eq_constraint(left_ty, bitvector_ty);
                        found = true
                    }
                }
                if !found {
                    todo!("Call constraint {:?}", call)
                }
            }
            WExprHighCall::StdUnary(unary) => todo!("Std unary"),
            WExprHighCall::StdBinary(binary) => {
                let a_ty = get_type(types, &binary.a)?;
                let b_ty = get_type(types, &binary.b)?;

                match binary.op {
                    IrStdBinaryOp::Eq
                    | IrStdBinaryOp::Ne
                    | IrStdBinaryOp::Lt
                    | IrStdBinaryOp::Le
                    | IrStdBinaryOp::Gt
                    | IrStdBinaryOp::Ge => {
                        // constrain the inputs to be of the same type
                        self.add_eq_constraint(a_ty, b_ty);
                        // constrain the output to be a Boolean
                        let bool_ty = self.partial_type_id(bool_type());
                        self.add_eq_constraint(left_ty, bool_ty);
                    }
                    IrStdBinaryOp::BitAnd
                    | IrStdBinaryOp::BitOr
                    | IrStdBinaryOp::BitXor
                    | IrStdBinaryOp::Shl
                    | IrStdBinaryOp::Shr
                    | IrStdBinaryOp::Add
                    | IrStdBinaryOp::Sub
                    | IrStdBinaryOp::Mul
                    | IrStdBinaryOp::Div
                    | IrStdBinaryOp::Rem => {
                        // constrain both inputs to output
                        self.add_eq_constraint(left_ty.clone(), a_ty);
                        self.add_eq_constraint(left_ty, b_ty);
                    }
                }
            }
        }
        Ok(())
    }
}

fn get_type(types: &BTreeMap<WIdent, WTypeId>, ident: &WIdent) -> Result<WTypeId, Error> {
    if let Some(global_ty) = types.get(ident) {
        return Ok(global_ty.clone());
    }

    Err(Error::new(
        ErrorType::UndefinedVariable(ident.name().to_string()),
        ident.wir_span(),
    ))
}
