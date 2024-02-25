use std::{collections::HashMap, vec};

use proc_macro2::Span;
use syn::{
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Block, Expr, Ident, Item, Local, Member, Pat, Path, Stmt, Type,
};

use crate::{util::extract_path_ident_mut, ErrorType, MachineError};

pub fn normalize_scope(items: &mut [Item]) -> Result<(), MachineError> {
    let mut visitor = Visitor {
        result: Ok(()),
        scope_idents: vec![],
        scope_num: 0,
        local_defs: vec![],
    };
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }

    visitor.result
}

struct Visitor {
    result: Result<(), MachineError>,
    scope_idents: Vec<HashMap<Ident, Vec<Ident>>>,
    scope_num: usize,
    local_defs: Vec<Local>,
}
impl VisitMut for Visitor {
    fn visit_impl_item_fn_mut(&mut self, impl_item_fn: &mut syn::ImplItemFn) {
        // delegate visit
        visit_mut::visit_impl_item_fn_mut(self, impl_item_fn);
        assert!(self.scope_idents.is_empty());

        // drain local defs to create let statements and add them at the start of block
        let mut stmts = Vec::from_iter(self.local_defs.drain(..).map(Stmt::Local));
        stmts.append(&mut impl_item_fn.block.stmts);
        impl_item_fn.block.stmts = stmts;
        assert!(self.local_defs.is_empty());
    }

    fn visit_block_mut(&mut self, block: &mut Block) {
        self.scope_num = self
            .scope_num
            .checked_add(1)
            .expect("Scope number should not overflow");
        // push scope idents
        self.scope_idents.push(HashMap::new());

        // process all statements
        let mut processed_stmts = Vec::new();
        for mut stmt in block.stmts.drain(..) {
            if let Stmt::Local(local) = stmt {
                if let Err(err) = self.process_local(local) {
                    self.result = Err(err);
                }
            } else {
                // visit the statement and add it to converted statements
                self.visit_stmt_mut(&mut stmt);
                processed_stmts.push(stmt);
            }
        }
        block.stmts = processed_stmts;

        // pop scope idents
        self.scope_idents.pop();
    }

    fn visit_ident_mut(&mut self, ident: &mut Ident) {
        // use reverse iteration to find the mapping in the innermost scope
        for scope in self.scope_idents.iter().rev() {
            if let Some(unique_ident_vec) = scope.get(ident) {
                // replace ident with the last unique ident created, the others are shadowed
                *ident = unique_ident_vec.last().unwrap().clone();
            }
        }
    }

    fn visit_path_mut(&mut self, path: &mut Path) {
        // only visit identifiers
        if let Some(ident) = extract_path_ident_mut(path) {
            self.visit_ident_mut(ident);
        }
    }

    fn visit_expr_struct_mut(&mut self, node: &mut syn::ExprStruct) {
        // handle shorthands gracefully: add the colon token first to convert from shorthand
        // after visiting the field (and potentially changing the expression path),
        // if it is possible to use shorthand, convert to it
        for mut el in node.fields.pairs_mut() {
            let it = el.value_mut();
            it.colon_token = Some(Default::default());
            self.visit_field_value_mut(it);
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
        // do not go into a member
    }
}

impl Visitor {
    fn process_local(&mut self, mut local: Local) -> Result<(), MachineError> {
        let mut pat = if let Pat::Type(ref mut pat_ty) = &mut local.pat {
            pat_ty.pat.as_mut()
        } else {
            &mut local.pat
        };

        let Pat::Ident(pat_ident) = &mut pat else {
            return Err(MachineError::new(
                ErrorType::SsaInternal(String::from(
                    "Unsupported non-ident pattern in scope normalization",
                )),
                pat.span(),
            ));
        };

        if pat_ident.subpat.is_some() {
            return Err(MachineError::new(
                ErrorType::SsaInternal(String::from(
                    "Unsupported pattern with subpattern in scope normalization",
                )),
                pat.span(),
            ));
        }

        if pat_ident.by_ref.is_some() {
            return Err(MachineError::new(
                ErrorType::SsaInternal(String::from(
                    "Unsupported pattern by reference in scope normalization",
                )),
                pat.span(),
            ));
        }

        let left_ident = pat_ident.ident.clone();

        let scope_num = self.scope_num;

        // find the vector of unique idents for the local in this scope
        let unique_ident_vec = self
            .scope_idents
            .last_mut()
            .unwrap()
            .entry(left_ident.clone())
            .or_default();

        // we can already have variables that will be shadowed in this scope
        // choose the unique ident name with that in mind
        let shadow_num = unique_ident_vec.len();

        let unique_ident = Ident::new(
            &format!("__mck_scope_{}_{}_{}", scope_num, shadow_num, left_ident),
            Span::call_site(),
        );
        // add ident to scope
        unique_ident_vec.push(unique_ident.clone());

        // the local does not contain initialization after construct normalization
        assert!(local.init.is_none());

        // assign unique ident to local and push the local to local defs
        pat_ident.ident = unique_ident;
        self.local_defs.push(local);
        Ok(())
    }
}
