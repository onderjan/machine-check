use std::{collections::HashMap, vec};

use proc_macro2::Span;
use syn::{visit_mut::VisitMut, Block, Expr, Ident, Item, Local, Member, Pat, Path, Stmt, Type};

use crate::{util::create_assign, MachineError};

pub fn normalize_scope(items: &mut [Item]) -> Result<(), MachineError> {
    let mut visitor = BlockVisitor {
        result: Ok(()),
        scope_idents: vec![],
        local_defs: vec![],
    };
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }

    visitor.result
}

struct BlockVisitor {
    result: Result<(), MachineError>,
    scope_idents: Vec<HashMap<Ident, Ident>>,
    local_defs: Vec<Local>,
}
impl VisitMut for BlockVisitor {
    fn visit_block_mut(&mut self, block: &mut Block) {
        let scope_num = self.scope_idents.len();
        self.scope_idents.push(HashMap::new());

        let mut original_stmts = Vec::new();
        original_stmts.append(&mut block.stmts);

        // process all statements
        for mut stmt in original_stmts {
            if let Stmt::Local(mut local) = stmt {
                let mut pat = if let Pat::Type(ref mut pat_ty) = &mut local.pat {
                    pat_ty.pat.as_mut()
                } else {
                    &mut local.pat
                };

                let Pat::Ident(pat_ident) = &mut pat else {
                    self.result = Err(MachineError(format!("Unsupported pattern: {:?}", pat)));
                    return;
                };

                if pat_ident.subpat.is_some() {
                    self.result = Err(MachineError(format!("Unsupported subpattern: {:?}", pat)));
                    return;
                }

                if pat_ident.by_ref.is_some() {
                    self.result = Err(MachineError(format!(
                        "Unsupported pattern by ref: {:?}",
                        pat
                    )));
                    return;
                }

                let left_ident = pat_ident.ident.clone();

                // create unique ident
                let unique_ident = Ident::new(
                    &format!("__mck_scope_{}_{}", scope_num, left_ident),
                    Span::call_site(),
                );
                // add ident to scope
                self.scope_idents
                    .last_mut()
                    .unwrap()
                    .insert(left_ident.clone(), unique_ident.clone());

                // only retain statement if it has initialization, convert it to assignment in that case
                if let Some(ref mut init) = local.init {
                    if init.diverge.is_some() {
                        self.result = Err(MachineError(format!(
                            "Diverging let not supported: {:?}",
                            init,
                        )));
                        return;
                    }
                    // remember to visit the right expression before adding the assignment to converted statements
                    let mut right_expr = init.expr.as_ref().clone();
                    self.visit_expr_mut(&mut right_expr);
                    block
                        .stmts
                        .push(create_assign(unique_ident.clone(), right_expr, true));
                    // drop init in local
                    local.init = None;
                }

                // assign unique ident to local and push the local to local defs
                pat_ident.ident = unique_ident;
                self.local_defs.push(local);
            } else {
                // visit the statement and add it to converted statements
                self.visit_stmt_mut(&mut stmt);
                block.stmts.push(stmt);
            }
        }

        self.scope_idents.pop();

        // add initializations of unique idents to outermost block
        if self.scope_idents.is_empty() {
            let mut stmts = vec![];
            let mut local_defs = vec![];
            local_defs.append(&mut self.local_defs);
            for local_def in local_defs {
                stmts.push(Stmt::Local(local_def));
            }
            stmts.append(&mut block.stmts);
            block.stmts = stmts;
        }
    }

    fn visit_ident_mut(&mut self, ident: &mut Ident) {
        // use reverse iteration to find the mapping in the innermost scope
        for scope in self.scope_idents.iter().rev() {
            if let Some(unique_ident) = scope.get(ident) {
                *ident = unique_ident.clone();
            }
        }
    }

    fn visit_field_mut(&mut self, node: &mut syn::Field) {
        for it in &mut node.attrs {
            self.visit_attribute_mut(it);
        }
        self.visit_visibility_mut(&mut node.vis);
        self.visit_field_mutability_mut(&mut node.mutability);
        // treat specially by not going into field
        self.visit_type_mut(&mut node.ty);
    }

    fn visit_expr_struct_mut(&mut self, node: &mut syn::ExprStruct) {
        // TODO: deduplicate with struct_rules
        for it in &mut node.attrs {
            self.visit_attribute_mut(it);
        }
        if let Some(it) = &mut node.qself {
            self.visit_qself_mut(it);
        }
        for mut el in node.fields.pairs_mut() {
            let it = el.value_mut();
            // handle shorthands gracefully: add the colon token first to convert from shorthand
            it.colon_token = Some(Default::default());
            self.visit_field_value_mut(it);
            // after visiting the field (and potentially changing the expression path),
            // if it is possible to use shorthand, convert to it
            if let Member::Named(member) = &it.member {
                if let Expr::Path(path) = &it.expr {
                    if path.path.is_ident(member) {
                        it.colon_token = None;
                    }
                }
            }
        }
        if let Some(it) = &mut node.rest {
            self.visit_expr_mut(it);
        }
    }

    fn visit_type_mut(&mut self, _: &mut Type) {
        // do not propagate
    }

    fn visit_member_mut(&mut self, _: &mut Member) {
        // do not go into the member
    }
    fn visit_path_mut(&mut self, path: &mut Path) {
        if path.leading_colon.is_none()
            && path.segments.len() == 1
            && path.segments[0].arguments.is_none()
        {
            // treat as identifier
            self.visit_ident_mut(&mut path.segments[0].ident);
        } else {
            // do not propagate
        }
    }
}
