use syn::Item;
use syn::{spanned::Spanned, Block, Expr, Stmt};

use crate::support::block_convert::{block_convert, TemporaryManager};
use crate::util::{create_assign, create_expr_path, extract_else_block_mut};

pub fn convert_to_tac(items: &mut [Item], temporary_manager: &mut TemporaryManager) {
    // conversion to three-address code does not raise errors
    // normalization is expected to reject all unsupported code beforehand
    let _ = block_convert(items, temporary_manager, |temporary_manager, block| {
        convert_block(temporary_manager, block);
        Ok(())
    });
}

fn convert_block(temporary_manager: &mut TemporaryManager, block: &mut Block) {
    let mut processed_stmts = Vec::new();
    let num_block_stmts = block.stmts.len();
    for (index, stmt) in block.stmts.drain(..).enumerate() {
        match stmt {
            Stmt::Expr(mut expr, semi) => {
                if semi.is_some()
                    || index + 1 != num_block_stmts
                    || matches!(expr, Expr::Path(_) | Expr::Struct(_) | Expr::Lit(_))
                {
                    // process expression without forced movement
                    // the newly created statements (for temporaries) will be added
                    // before the (possibly changed) processed statement
                    convert_expr(temporary_manager, &mut processed_stmts, &mut expr);
                    processed_stmts.push(Stmt::Expr(expr, semi));
                } else {
                    // force movement to ensure there is only a path, struct or literal in return position
                    move_through_temp(temporary_manager, &mut processed_stmts, &mut expr);
                    processed_stmts.push(Stmt::Expr(expr, semi));
                }
            }
            Stmt::Local(_) => {
                // just retain
                processed_stmts.push(stmt);
            }
            _ => {
                panic!("Unexpected statement type in three-address-code conversion");
            }
        }
    }
    block.stmts = processed_stmts;
}

fn convert_expr(
    temporary_manager: &mut TemporaryManager,
    assign_stmts: &mut Vec<Stmt>,
    expr: &mut Expr,
) {
    match expr {
        syn::Expr::Path(_) | syn::Expr::Lit(_) => {
            // do nothing, paths and literals are not moved in our SSA
        }
        syn::Expr::Field(field) => {
            // move base
            move_through_temp(temporary_manager, assign_stmts, &mut field.base);
        }
        syn::Expr::Index(expr_index) => {
            // base cannot be moved, move index
            move_through_temp(temporary_manager, assign_stmts, &mut expr_index.index);
        }
        syn::Expr::Paren(paren) => {
            // move statement in parentheses
            move_through_temp(temporary_manager, assign_stmts, &mut paren.expr);
            // remove parentheses
            *expr = (*paren.expr).clone();
        }
        syn::Expr::Reference(reference) => {
            // do not move field expression
            let mut move_expr = reference.expr.as_mut();
            if let Expr::Field(expr_field) = move_expr {
                move_expr = &mut expr_field.base;
            }
            // move expression
            move_through_temp(temporary_manager, assign_stmts, move_expr);
        }
        syn::Expr::Call(call) => {
            // move call function expression and arguments
            move_through_temp(temporary_manager, assign_stmts, &mut call.func);
            for arg in &mut call.args {
                move_through_temp(temporary_manager, assign_stmts, arg);
            }
        }
        syn::Expr::Assign(assign) => {
            match assign.left.as_mut() {
                Expr::Path(_) => {
                    match assign.right.as_mut() {
                        Expr::Block(_) => {
                            // force movement
                            move_through_temp(
                                temporary_manager,
                                assign_stmts,
                                assign.right.as_mut(),
                            );
                        }
                        _ => {
                            // apply translation to right-hand expression without forced movement
                            convert_expr(temporary_manager, assign_stmts, assign.right.as_mut());
                        }
                    }
                }
                Expr::Index(_) => {
                    // force movement of right expression
                    convert_expr(temporary_manager, assign_stmts, assign.right.as_mut());
                }
                _ => {
                    panic!("Unexpected assignment left side in three-address-code conversion")
                }
            }
        }
        syn::Expr::Struct(expr_struct) => {
            // move field values
            for field in &mut expr_struct.fields {
                move_through_temp(temporary_manager, assign_stmts, &mut field.expr);
            }
            if expr_struct.rest.is_some() {
                panic!("Unexpected struct rest in three-address-code conversion");
            }
        }
        syn::Expr::Block(expr_block) => {
            // process the block
            convert_block(temporary_manager, &mut expr_block.block);
        }
        syn::Expr::If(expr_if) => {
            // move condition if it is not special
            let mut should_move = true;
            if let Expr::Call(cond_expr_call) = expr_if.cond.as_mut() {
                if let Expr::Path(cond_expr_path) = cond_expr_call.func.as_ref() {
                    if cond_expr_path.path.leading_colon.is_some() {
                        let segments = &cond_expr_path.path.segments;

                        // TODO: integrate the special conditions better
                        if segments.len() == 4
                            && &segments[0].ident.to_string() == "mck"
                            && &segments[1].ident.to_string() == "concr"
                            && &segments[2].ident.to_string() == "Test"
                            && &segments[3].ident.to_string() == "into_bool"
                        {
                            // only move the inside
                            should_move = false;
                            for arg in cond_expr_call.args.iter_mut() {
                                move_through_temp(temporary_manager, assign_stmts, arg);
                            }
                        }
                    }
                }
            }
            if should_move {
                move_through_temp(temporary_manager, assign_stmts, &mut expr_if.cond);
            }
            // process then and else blocks
            convert_block(temporary_manager, &mut expr_if.then_branch);
            convert_block(
                temporary_manager,
                extract_else_block_mut(&mut expr_if.else_branch).expect("Expected else block"),
            );
        }
        _ => {
            panic!("Unexpected expression type in three-address-code conversion",);
        }
    }
}

pub(super) fn move_through_temp(
    temporary_manager: &mut TemporaryManager,
    assign_stmts: &mut Vec<Stmt>,
    expr: &mut Expr,
) {
    // process the expression first before moving it through temporary
    match expr {
        syn::Expr::Path(_) | syn::Expr::Lit(_) => {
            // do nothing, paths and literals are not moved in our SSA
            return;
        }
        syn::Expr::Paren(paren) => {
            // move statement in parentheses
            move_through_temp(temporary_manager, assign_stmts, &mut paren.expr);
            // remove parentheses
            *expr = (*paren.expr).clone();
            return;
        }
        _ => {
            // process the expression normally
            // so that nested expressions are properly converted to SSA
            convert_expr(temporary_manager, assign_stmts, expr);
        }
    }

    // create a temporary variable
    let tmp_ident = temporary_manager.create_temporary_ident(expr.span(), None);
    // add assignment statement; the temporary is only assigned to once here
    assign_stmts.push(create_assign(tmp_ident.clone(), expr.clone(), true));

    // change expr to the temporary variable path
    *expr = create_expr_path(tmp_ident.into());
}
