mod convert_mutable;
mod infer_types;
mod normalize_scope;

use proc_macro2::Span;
use syn::{visit_mut::VisitMut, Block, Expr, Ident, Item, Stmt};

use crate::{
    util::{create_assign, create_expr_path, create_let_bare},
    MachineDescription, MachineError,
};

pub(crate) fn create_concrete_machine(
    mut items: Vec<Item>,
) -> Result<MachineDescription, MachineError> {
    normalize_scope::normalize_scope(&mut items)?;
    convert_mutable::convert_mutable(&mut items)?;

    // apply linear SSA to each block using a visitor
    struct Visitor(Result<(), MachineError>);
    impl VisitMut for Visitor {
        fn visit_block_mut(&mut self, block: &mut Block) {
            // start with zero temp counter in an outer-level block
            let result = Outer::new().apply_to_block(block, true);
            if let Err(err) = result {
                self.0 = Err(err);
            }
            // do not delegate, the representation should not contain
            // any nested blocks anyway after translation
        }
    }
    let mut visitor = Visitor(Ok(()));
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }
    visitor.0?;

    infer_types::infer_types(&mut items)?;

    Ok(MachineDescription { items })
}

struct Outer {
    next_temp_counter: u32,
    created_temporaries: Vec<Ident>,
}

impl Outer {
    fn new() -> Self {
        Outer {
            next_temp_counter: 0,
            created_temporaries: vec![],
        }
    }

    fn apply_to_block(&mut self, block: &mut Block, outer: bool) -> Result<(), MachineError> {
        // use the same temp counter
        let mut translator = BlockTranslator {
            translated_stmts: Vec::new(),
            outer: self,
        };
        // apply linear SSA to statements one by one
        for stmt in &block.stmts {
            translator.apply_to_stmt(stmt.clone())?;
        }

        let mut stmts = translator.translated_stmts;

        if outer {
            // do not add types to temporaries, they will be inferred later
            block.stmts = self
                .created_temporaries
                .iter()
                .map(|tmp_ident| create_let_bare(tmp_ident.clone(), None))
                .collect();
            block.stmts.append(&mut stmts);
        } else {
            block.stmts = stmts;
        }
        Ok(())
    }
}

struct BlockTranslator<'a> {
    translated_stmts: Vec<Stmt>,
    outer: &'a mut Outer,
}

impl<'a> BlockTranslator<'a> {
    fn apply_to_stmt(&mut self, mut stmt: Stmt) -> Result<(), MachineError> {
        match stmt {
            Stmt::Expr(ref mut expr, _) => {
                // apply translation to expression without forced movement
                self.apply_to_expr(expr)?;
            }
            Stmt::Local(_) => {
                // do nothing
            }
            _ => {
                return Err(MachineError(format!(
                    "Statement type {:?} not supported",
                    stmt
                )))
            }
        }
        self.translated_stmts.push(stmt);
        Ok(())
    }

    fn apply_to_expr(&mut self, expr: &mut Expr) -> Result<(), MachineError> {
        match expr {
            syn::Expr::Path(_) | syn::Expr::Lit(_) => {
                // do nothing, paths and literals are not moved in our SSA
            }
            syn::Expr::Field(field) => {
                // move base
                self.move_through_temp(&mut field.base)?;
            }
            syn::Expr::Paren(paren) => {
                // move statement in parentheses
                self.move_through_temp(&mut paren.expr)?;
                // remove parentheses
                *expr = (*paren.expr).clone();
            }
            syn::Expr::Call(call) => {
                // move call function expression and arguments
                self.move_through_temp(&mut call.func)?;
                for arg in &mut call.args {
                    self.move_through_temp(arg)?;
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
                        self.move_through_temp(assign.right.as_mut())?;
                    }
                    _ => {
                        // apply translation to right-hand expression without forced movement
                        self.apply_to_expr(assign.right.as_mut())?;
                    }
                }
            }
            syn::Expr::Struct(expr_struct) => {
                // move field values
                for field in &mut expr_struct.fields {
                    self.move_through_temp(&mut field.expr)?;
                }
                if expr_struct.rest.is_some() {
                    return Err(MachineError("Struct rest not supported".to_string()));
                }
            }
            syn::Expr::Block(expr_block) => {
                // apply in new scope
                self.outer.apply_to_block(&mut expr_block.block, false)?;
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
                                    self.move_through_temp(arg)?;
                                }
                            }
                        }
                    }
                }
                if should_move {
                    self.move_through_temp(&mut expr_if.cond)?;
                }
                // apply to then-branch
                self.outer.apply_to_block(&mut expr_if.then_branch, false)?;
                // apply to else-branch if it exists
                if let Some(else_branch) = &mut expr_if.else_branch {
                    // should be a block, if expression, or if-let expression
                    let else_branch = else_branch.1.as_mut();

                    if matches!(else_branch, Expr::If(_) | Expr::Block(_)) {
                        self.apply_to_expr(else_branch)?;
                    } else {
                        return Err(MachineError(format!(
                            "Unexpected expression type of else branch: {:?}",
                            else_branch
                        )));
                    }
                }
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

    fn move_through_temp(&mut self, expr: &mut Expr) -> Result<(), MachineError> {
        match expr {
            syn::Expr::Path(_) | syn::Expr::Lit(_) => {
                // do nothing, paths and literals are not moved in our SSA
                return Ok(());
            }
            syn::Expr::Paren(paren) => {
                // move statement in parentheses
                self.move_through_temp(&mut paren.expr)?;
                // remove parentheses
                *expr = (*paren.expr).clone();
                return Ok(());
            }
            _ => {
                // apply translation to expression
                // so that nested expressions are properly converted to SSA
                self.apply_to_expr(expr)?;
            }
        }

        // create a temporary variable
        let tmp_ident = Ident::new(
            format!("__mck_ssa_{}", self.outer.next_temp_counter).as_str(),
            Span::call_site(),
        );
        self.outer.next_temp_counter += 1;

        // add to created temporaries, they will get their bare let statements created later
        self.outer.created_temporaries.push(tmp_ident.clone());
        // add assignment statement; the temporary is only assigned to once here
        self.translated_stmts
            .push(create_assign(tmp_ident.clone(), expr.clone(), true));

        // change expr to the temporary variable path
        *expr = create_expr_path(tmp_ident.into());
        Ok(())
    }
}
