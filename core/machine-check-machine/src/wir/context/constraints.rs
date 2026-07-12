use machine_check_common::ir_common::IrStdBinaryOp;
use syn::{Path, Type, TypePath};
use syn_path::path;

use crate::{
    into_wir::Error,
    wir::{
        context::{bitvector_type, bool_type},
        WBlock, WExpr, WExprHighCall, WIndexedExpr, WIndexedIdent, WMacroableStmt,
        WPartialArgument, WTacLocal, WTypeId, ZTac,
    },
};

impl super::WContext {
    pub(super) fn add_block_constraints(
        &mut self,
        locals: &Vec<WTacLocal<WTypeId>>,
        block: &WBlock<ZTac>,
    ) -> Result<(), Error> {
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

                    let left_ty = locals
                        .iter()
                        .find(|e| &e.ident == left)
                        .expect("Local should be found")
                        .ty
                        .clone();

                    match right {
                        WExpr::Move(wident) => todo!("Move"),
                        WExpr::Call(call) => {
                            eprintln!("Call");
                            match call {
                                WExprHighCall::Call(call) => {
                                    let mut found = false;
                                    if call.fn_path.leading_colon.is_some()
                                        && call.fn_path.segments.len() == 3
                                    {
                                        let segments = &call.fn_path.segments;
                                        if segments[0].ident.name() == "machine_check"
                                            && segments[1].ident.name() == "Bitvector"
                                            && segments[2].ident.name() == "new"
                                        {
                                            let mut width = None;
                                            if let Some(generics) = &segments[1].generics {
                                                if generics.arguments.len() == 1 {
                                                    if let WPartialArgument::Uint(
                                                        width_arg,
                                                        _span,
                                                    ) = generics.arguments[0]
                                                    {
                                                        width = Some(width_arg)
                                                    }
                                                }
                                            }
                                            // constrain the output to be a bitvector of the given width
                                            let bitvector_ty =
                                                self.partial_type_id(bitvector_type(width));
                                            self.add_eq_constraint(left_ty, bitvector_ty);
                                            found = true
                                        }
                                    }
                                    if !found {
                                        todo!("Call {:?}", call)
                                    }
                                }
                                WExprHighCall::StdUnary(unary) => todo!("Std unary"),
                                WExprHighCall::StdBinary(binary) => {
                                    let a_ty = locals
                                        .iter()
                                        .find(|e| e.ident == binary.a)
                                        .expect("Local should be found")
                                        .ty
                                        .clone();

                                    let b_ty = locals
                                        .iter()
                                        .find(|e| e.ident == binary.b)
                                        .expect("Local should be found")
                                        .ty
                                        .clone();
                                    match binary.op {
                                        IrStdBinaryOp::Eq => {
                                            // constrain the inputs to be of the same type
                                            self.add_eq_constraint(a_ty, b_ty);
                                            // constrain the output to be a Boolean
                                            let bool_ty = self.partial_type_id(bool_type());
                                            self.add_eq_constraint(left_ty, bool_ty);
                                        }
                                        IrStdBinaryOp::Add => {
                                            // constrain both inputs to output
                                            self.add_eq_constraint(left_ty.clone(), a_ty);
                                            self.add_eq_constraint(left_ty, b_ty);
                                        }
                                        _ => todo!("Std binary"),
                                    }
                                }
                            }
                        }
                        WExpr::Field(wexpr_field) => {
                            eprintln!("Field");
                        }
                        WExpr::Struct(expr_struct) => {
                            let struct_ty = self.type_id(&Type::Path(TypePath {
                                qself: None,
                                path: Path::from(expr_struct.type_path.clone()),
                            }))?;
                            self.add_eq_constraint(left_ty, struct_ty);
                        }
                        WExpr::Reference(wexpr_reference) => todo!("Reference"),
                        WExpr::Lit(lit, _) => todo!("Literal"),
                    }
                }
                WMacroableStmt::If(stmt_if) => {
                    eprintln!("Should add constraints for if {:#?}", stmt_if);
                    self.add_block_constraints(locals, &stmt_if.then_block)?;
                    self.add_block_constraints(locals, &stmt_if.else_block)?;
                }
                WMacroableStmt::PanicMacro(wstmt_panic_macro) => {
                    todo!("Constraints for panic macro")
                }
            }
        }
        Ok(())
    }
}
