use proc_macro2::Span;
use syn::{visit_mut::VisitMut, Block, Expr, Ident, Item, Stmt};

use crate::{
    util::{create_assign, create_expr_path, create_let_bare, extract_else_block_mut},
    MachineError,
};

pub fn convert_to_tac(items: &mut [Item]) -> Result<(), MachineError> {
    // normalize to three-address code by adding temporaries
    let mut visitor = Visitor { result: Ok(()) };
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }
    visitor.result
}

struct Visitor {
    result: Result<(), MachineError>,
}
impl VisitMut for Visitor {
    fn visit_impl_item_fn_mut(&mut self, impl_item_fn: &mut syn::ImplItemFn) {
        let result = process_impl_item_fn(impl_item_fn);
        if let Err(err) = result {
            self.result = Err(err);
        }
    }
}

fn process_impl_item_fn(impl_item_fn: &mut syn::ImplItemFn) -> Result<(), MachineError> {
    let mut converter = Converter {
        next_temp_counter: 0,
        created_temporaries: vec![],
    };
    converter.process_block(&mut impl_item_fn.block)?;

    // prefix the function block with newly created temporaries
    // do not add types to temporaries, they will be inferred later
    let mut stmts: Vec<Stmt> = converter
        .created_temporaries
        .iter()
        .map(|tmp_ident| create_let_bare(tmp_ident.clone(), None))
        .collect();
    stmts.append(&mut impl_item_fn.block.stmts);
    impl_item_fn.block.stmts.append(&mut stmts);

    Ok(())
}

struct Converter {
    next_temp_counter: u32,
    created_temporaries: Vec<Ident>,
}

impl Converter {
    fn process_block(&mut self, block: &mut Block) -> Result<(), MachineError> {
        let mut processed_stmts = Vec::new();
        for mut stmt in block.stmts.drain(..) {
            match stmt {
                Stmt::Expr(ref mut expr, _) => {
                    // process expression without forced movement
                    // the newly created statements (for temporaries) will be added
                    // before the (possibly changed) processed statement
                    self.process_expr(&mut processed_stmts, expr)?;
                    processed_stmts.push(stmt);
                }
                Stmt::Local(_) => {
                    // just retain
                    processed_stmts.push(stmt);
                }
                _ => {
                    return Err(MachineError(format!(
                        "Statement type {:?} not supported",
                        stmt
                    )))
                }
            }
        }
        block.stmts = processed_stmts;
        Ok(())
    }

    fn process_expr(
        &mut self,
        assign_stmts: &mut Vec<Stmt>,
        expr: &mut Expr,
    ) -> Result<(), MachineError> {
        match expr {
            syn::Expr::Path(_) | syn::Expr::Lit(_) => {
                // do nothing, paths and literals are not moved in our SSA
            }
            syn::Expr::Field(field) => {
                // move base
                self.move_through_temp(assign_stmts, &mut field.base)?;
            }
            syn::Expr::Paren(paren) => {
                // move statement in parentheses
                self.move_through_temp(assign_stmts, &mut paren.expr)?;
                // remove parentheses
                *expr = (*paren.expr).clone();
            }
            syn::Expr::Call(call) => {
                // move call function expression and arguments
                self.move_through_temp(assign_stmts, &mut call.func)?;
                for arg in &mut call.args {
                    self.move_through_temp(assign_stmts, arg)?;
                }
            }
            syn::Expr::Assign(assign) => {
                if !matches!(assign.left.as_ref(), Expr::Path(_)) {
                    return Err(MachineError(format!(
                        "Non-path left not supported in assignment: {:?}",
                        assign.left,
                    )));
                }

                match assign.right.as_mut() {
                    Expr::Block(_) => {
                        // force movement
                        self.move_through_temp(assign_stmts, assign.right.as_mut())?;
                    }
                    _ => {
                        // apply translation to right-hand expression without forced movement
                        self.process_expr(assign_stmts, assign.right.as_mut())?;
                    }
                }
            }
            syn::Expr::Struct(expr_struct) => {
                // move field values
                for field in &mut expr_struct.fields {
                    self.move_through_temp(assign_stmts, &mut field.expr)?;
                }
                if expr_struct.rest.is_some() {
                    return Err(MachineError("Struct rest not supported".to_string()));
                }
            }
            syn::Expr::Block(expr_block) => {
                // process the block
                self.process_block(&mut expr_block.block)?;
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
                                    self.move_through_temp(assign_stmts, arg)?;
                                }
                            }
                        }
                    }
                }
                if should_move {
                    self.move_through_temp(assign_stmts, &mut expr_if.cond)?;
                }
                // process then and else blocks
                self.process_block(&mut expr_if.then_branch)?;
                self.process_block(
                    extract_else_block_mut(&mut expr_if.else_branch).expect("Expected else block"),
                )?;
            }
            _ => {
                return Err(MachineError(format!(
                    "Expression type not supported: {:?}",
                    expr
                )));
            }
        }
        Ok(())
    }

    fn move_through_temp(
        &mut self,
        assign_stmts: &mut Vec<Stmt>,
        expr: &mut Expr,
    ) -> Result<(), MachineError> {
        // process the expression first before moving it through temporary
        match expr {
            syn::Expr::Path(_) | syn::Expr::Lit(_) => {
                // do nothing, paths and literals are not moved in our SSA
                return Ok(());
            }
            syn::Expr::Paren(paren) => {
                // move statement in parentheses
                self.move_through_temp(assign_stmts, &mut paren.expr)?;
                // remove parentheses
                *expr = (*paren.expr).clone();
                return Ok(());
            }
            _ => {
                // process the expression normally
                // so that nested expressions are properly converted to SSA
                self.process_expr(assign_stmts, expr)?;
            }
        }

        // create a temporary variable
        let tmp_ident = Ident::new(
            format!("__mck_tac_{}", self.next_temp_counter).as_str(),
            Span::call_site(),
        );
        self.next_temp_counter = self
            .next_temp_counter
            .checked_add(1)
            .expect("Temp counter should not overflow");

        // add to created temporaries, they will get their let statements created later
        self.created_temporaries.push(tmp_ident.clone());
        // add assignment statement; the temporary is only assigned to once here
        assign_stmts.push(create_assign(tmp_ident.clone(), expr.clone(), true));

        // change expr to the temporary variable path
        *expr = create_expr_path(tmp_ident.into());
        Ok(())
    }
}
