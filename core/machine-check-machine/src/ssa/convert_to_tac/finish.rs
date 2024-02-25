use syn::{spanned::Spanned, Block, Expr, ExprAssign, ExprInfer, Ident, Stmt};
use syn_path::path;

use crate::{
    util::{
        create_expr_call, create_expr_ident, create_expr_path, create_expr_reference,
        extract_else_block_mut, ArgType,
    },
    ErrorType, MachineError,
};

impl super::Converter<'_> {
    pub(super) fn finish_block(&mut self, block: &mut Block) -> Result<(), MachineError> {
        block.stmts = self.finish_stmts(std::mem::take(&mut block.stmts))?;
        Ok(())
    }

    fn finish_stmts(&mut self, stmts: Vec<Stmt>) -> Result<Vec<Stmt>, MachineError> {
        let mut processed_stmts = Vec::new();
        for mut stmt in stmts {
            let added_stmts = self.finish_stmt(&mut stmt)?;
            processed_stmts.extend(self.finish_stmts(added_stmts)?);
            processed_stmts.push(stmt);
        }
        Ok(processed_stmts)
    }

    fn finish_stmt(&mut self, stmt: &mut Stmt) -> Result<Vec<Stmt>, MachineError> {
        let mut added_stmts = Vec::new();
        match stmt {
            Stmt::Expr(expr, _semi) => {
                match expr {
                    syn::Expr::Path(_) | syn::Expr::Struct(_) => {
                        // OK
                    }
                    syn::Expr::Block(ref mut expr_block) => {
                        // finish the block
                        self.finish_block(&mut expr_block.block)?;
                    }
                    syn::Expr::If(ref mut expr_if) => {
                        // finish then and else blocks
                        self.finish_block(&mut expr_if.then_branch)?;
                        self.finish_block(
                            extract_else_block_mut(&mut expr_if.else_branch)
                                .expect("Expected else block"),
                        )?;
                    }
                    syn::Expr::Assign(ref mut expr_assign) => {
                        // convert indexing to ReadWrite
                        if let Expr::Index(right_expr) = expr_assign.right.as_mut() {
                            // perform forced movement on index
                            self.move_through_temp(&mut added_stmts, right_expr.index.as_mut())?;

                            let right_base = std::mem::replace(
                                right_expr.expr.as_mut(),
                                Expr::Infer(ExprInfer {
                                    attrs: vec![],
                                    underscore_token: Default::default(),
                                }),
                            );
                            let right_index = std::mem::replace(
                                right_expr.index.as_mut(),
                                Expr::Infer(ExprInfer {
                                    attrs: vec![],
                                    underscore_token: Default::default(),
                                }),
                            );

                            // create a temporary variable for reference to right base
                            let tmp_ident = Ident::new(
                                format!("__mck_tac_{}", self.get_and_increment_temp_counter())
                                    .as_str(),
                                right_base.span(),
                            );
                            self.created_temporaries.push(tmp_ident.clone());

                            // assign reference to the array
                            added_stmts.push(Stmt::Expr(
                                Expr::Assign(ExprAssign {
                                    attrs: vec![],
                                    left: Box::new(create_expr_ident(tmp_ident.clone())),
                                    eq_token: Default::default(),
                                    right: Box::new(create_expr_reference(false, right_base)),
                                }),
                                Some(Default::default()),
                            ));
                            // the read call consumes the reference and index
                            let read_call = create_expr_call(
                                create_expr_path(path!(::mck::forward::ReadWrite::read)),
                                vec![
                                    (ArgType::Normal, create_expr_ident(tmp_ident)),
                                    (ArgType::Normal, right_index),
                                ],
                            );
                            expr_assign.right = Box::new(read_call);
                        };

                        if let Expr::Index(left_expr) = expr_assign.left.as_mut() {
                            // perform forced movement on index and right
                            // perform forced movement on index
                            self.move_through_temp(&mut added_stmts, left_expr.index.as_mut())?;
                            self.move_through_temp(&mut added_stmts, &mut expr_assign.right)?;
                            // convert to write
                            // the base must be without side-effects
                            let left_base = std::mem::replace(
                                left_expr.expr.as_mut(),
                                Expr::Infer(ExprInfer {
                                    attrs: vec![],
                                    underscore_token: Default::default(),
                                }),
                            );
                            let left_index = std::mem::replace(
                                left_expr.index.as_mut(),
                                Expr::Infer(ExprInfer {
                                    attrs: vec![],
                                    underscore_token: Default::default(),
                                }),
                            );
                            let right = std::mem::replace(
                                expr_assign.right.as_mut(),
                                Expr::Infer(ExprInfer {
                                    attrs: vec![],
                                    underscore_token: Default::default(),
                                }),
                            );

                            if !matches!(left_base, Expr::Path(_)) {
                                return Err(MachineError::new(
                                    ErrorType::SsaInternal(String::from(
                                        "Non-path base in left-hand indexing when finishing three-address-code conversion",
                                    )),
                                    left_base.span(),
                                ));
                            }

                            // create a temporary variable for reference to left base
                            let tmp_ident = Ident::new(
                                format!("__mck_tac_{}", self.get_and_increment_temp_counter())
                                    .as_str(),
                                left_base.span(),
                            );
                            self.created_temporaries.push(tmp_ident.clone());
                            // assign reference to the array
                            added_stmts.push(Stmt::Expr(
                                Expr::Assign(ExprAssign {
                                    attrs: vec![],
                                    left: Box::new(create_expr_ident(tmp_ident.clone())),
                                    eq_token: Default::default(),
                                    right: Box::new(create_expr_reference(
                                        false,
                                        left_base.clone(),
                                    )),
                                }),
                                Some(Default::default()),
                            ));

                            // the base is let through
                            let write_call = create_expr_call(
                                create_expr_path(path!(::mck::forward::ReadWrite::write)),
                                vec![
                                    (ArgType::Normal, create_expr_ident(tmp_ident)),
                                    (ArgType::Normal, left_index),
                                    (ArgType::Normal, right),
                                ],
                            );
                            expr_assign.left = Box::new(left_base);
                            expr_assign.right = Box::new(write_call);
                        }
                    }
                    _ => {
                        return Err(MachineError::new(ErrorType::SsaInternal(String::from(
                            "Unexpected expression type when finishing three-address-code conversion")),
                            expr.span()
                        ));
                    }
                }
            }
            Stmt::Local(_) => {
                // just retain
            }
            _ => {
                return Err(MachineError::new(
                    ErrorType::SsaInternal(String::from(
                        "Unexpected statement type when finishing three-address-code conversion",
                    )),
                    stmt.span(),
                ));
            }
        }
        Ok(added_stmts)
    }
}
