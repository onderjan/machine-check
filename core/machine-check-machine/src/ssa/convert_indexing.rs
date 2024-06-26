use syn::Item;

use crate::{
    support::block_convert::{block_convert, TemporaryManager},
    MachineError,
};
use syn::{spanned::Spanned, Block, Expr, ExprAssign, ExprInfer, Stmt};
use syn_path::path;

use crate::{
    util::{
        create_expr_call, create_expr_ident, create_expr_path, create_expr_reference,
        extract_else_block_mut, ArgType,
    },
    ErrorType,
};

use super::convert_to_tac::move_through_temp;

pub fn convert_indexing(
    items: &mut [Item],
    temporary_manager: &mut TemporaryManager,
) -> Result<(), MachineError> {
    block_convert(items, temporary_manager, convert_block)
}

fn convert_block(
    temporary_manager: &mut TemporaryManager,
    block: &mut Block,
) -> Result<(), MachineError> {
    block.stmts = convert_stmts(temporary_manager, std::mem::take(&mut block.stmts))?;
    Ok(())
}

fn convert_stmts(
    temporary_manager: &mut TemporaryManager,
    stmts: Vec<Stmt>,
) -> Result<Vec<Stmt>, MachineError> {
    let mut processed_stmts = Vec::new();
    for mut stmt in stmts {
        let added_stmts = convert_stmt(temporary_manager, &mut stmt)?;
        processed_stmts.extend(convert_stmts(temporary_manager, added_stmts)?);
        processed_stmts.push(stmt);
    }
    Ok(processed_stmts)
}

fn convert_stmt(
    temporary_manager: &mut TemporaryManager,
    stmt: &mut Stmt,
) -> Result<Vec<Stmt>, MachineError> {
    let mut added_stmts = Vec::new();
    match stmt {
        Stmt::Expr(expr, _semi) => {
            match expr {
                syn::Expr::Path(_) | syn::Expr::Struct(_) => {
                    // OK
                }
                syn::Expr::Block(ref mut expr_block) => {
                    // convert the block
                    convert_block(temporary_manager, &mut expr_block.block)?;
                }
                syn::Expr::If(ref mut expr_if) => {
                    // convert then and else blocks
                    convert_block(temporary_manager, &mut expr_if.then_branch)?;
                    convert_block(
                        temporary_manager,
                        extract_else_block_mut(&mut expr_if.else_branch)
                            .expect("Expected else block"),
                    )?;
                }
                syn::Expr::Assign(ref mut expr_assign) => {
                    convert_assign(temporary_manager, expr_assign, &mut added_stmts)?;
                }
                syn::Expr::Call(_) => {
                    // no need to convert, the parameters are idents
                }
                _ => panic!(
                    "Unexpected expression type in indexing conversion ({:?})",
                    expr.span()
                ),
            }
        }
        Stmt::Local(_) => {
            // just retain
        }
        _ => {
            panic!(
                "Unexpected statement type in indexing conversion ({:?})",
                stmt.span()
            );
        }
    }
    Ok(added_stmts)
}

fn convert_assign(
    temporary_manager: &mut TemporaryManager,
    expr_assign: &mut ExprAssign,
    added_stmts: &mut Vec<Stmt>,
) -> Result<(), MachineError> {
    // convert indexing to ReadWrite
    if let Expr::Index(right_expr) = expr_assign.right.as_mut() {
        // perform forced movement on index
        move_through_temp(temporary_manager, added_stmts, right_expr.index.as_mut());

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
        let tmp_ident = temporary_manager.create_temporary_ident(right_base.span(), None);

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
        move_through_temp(temporary_manager, added_stmts, left_expr.index.as_mut());
        move_through_temp(temporary_manager, added_stmts, &mut expr_assign.right);
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
            // we do not support non-path bases in assignee expression
            // this would be hard to detect when normalizing, so detect it here
            return Err(MachineError::new(
                ErrorType::UnsupportedConstruct(String::from(
                    "Non-path base in assignee expression",
                )),
                left_base.span(),
            ));
        }

        // create a temporary variable for reference to left base
        let tmp_ident = temporary_manager.create_temporary_ident(left_base.span(), None);
        // assign reference to the array
        added_stmts.push(Stmt::Expr(
            Expr::Assign(ExprAssign {
                attrs: vec![],
                left: Box::new(create_expr_ident(tmp_ident.clone())),
                eq_token: Default::default(),
                right: Box::new(create_expr_reference(false, left_base.clone())),
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
    Ok(())
}
