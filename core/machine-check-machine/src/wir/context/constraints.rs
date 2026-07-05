use machine_check_common::ir_common::IrStdBinaryOp;
use syn::{Path, Type, TypePath};
use syn_path::path;

use crate::{
    into_wir::Error,
    wir::{
        context::{bitvector_type, bool_type},
        WBlock, WExpr, WExprHighCall, WIndexedExpr, WIndexedIdent, WMacroableStmt, WTacLocal,
        WTypeId, ZTac,
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
                                    let bitvector_new_path = path!(::machine_check::Bitvector::new);
                                    let fn_path = Path::from(call.fn_path.clone());
                                    if fn_path == bitvector_new_path {
                                        // constrain the output to be a bitvector
                                        let bitvector_ty =
                                            self.partial_type_id(bitvector_type(None));
                                        self.add_eq_constraint(left_ty, bitvector_ty);

                                        eprintln!("Bitvector new");
                                    } else {
                                        todo!("Call")
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
