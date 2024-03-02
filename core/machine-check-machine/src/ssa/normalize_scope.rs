use std::{collections::HashMap, vec};

use syn::{
    visit_mut::{self, VisitMut},
    Block, Expr, FnArg, Ident, Item, Local, Member, Pat, Path, Stmt, Type,
};

use crate::{support::local::construct_prefixed_ident, util::extract_path_ident_mut};

pub fn normalize_scope(items: &mut [Item]) {
    let mut visitor = Visitor {
        scope_idents: vec![],
        scope_num: 0,
        local_defs: vec![],
    };
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }
}

struct Visitor {
    scope_idents: Vec<HashMap<Ident, Vec<Ident>>>,
    scope_num: usize,
    local_defs: Vec<Local>,
}
impl VisitMut for Visitor {
    fn visit_impl_item_fn_mut(&mut self, impl_item_fn: &mut syn::ImplItemFn) {
        // add non-self parameters to scope idents
        self.scope_num = self
            .scope_num
            .checked_add(1)
            .expect("Scope number should not overflow");
        let mut param_ident_map = HashMap::new();
        for param in impl_item_fn.sig.inputs.iter() {
            if let FnArg::Typed(param) = param {
                let Pat::Ident(pat_ident) = param.pat.as_ref() else {
                    panic!("Function parameters should be identifers in scope normalization");
                };
                let param_ident = &pat_ident.ident;
                let unique_ident =
                    construct_prefixed_ident(&format!("scope_{}_0", self.scope_num), param_ident);
                // the map contains keys with original idents and values representing unique idents
                // the idents can be shadowed within the same scope, so the last unique ident is taken
                param_ident_map.insert(param_ident.clone(), vec![unique_ident]);
            }
        }
        self.scope_idents.push(param_ident_map);

        // delegate visit
        visit_mut::visit_impl_item_fn_mut(self, impl_item_fn);

        // remove the parameters scope
        assert_eq!(self.scope_idents.len(), 1);
        self.scope_idents.clear();

        // put the local statements at the start of the function block
        impl_item_fn.block.stmts = Vec::from_iter(
            self.local_defs
                .drain(..)
                .map(Stmt::Local)
                .chain(impl_item_fn.block.stmts.drain(..)),
        );
    }

    fn visit_block_mut(&mut self, block: &mut Block) {
        self.scope_num = self
            .scope_num
            .checked_add(1)
            .expect("Scope number should not overflow");
        // push scope, currenly with no idents
        self.scope_idents.push(HashMap::new());

        // process all statements
        let mut processed_stmts = Vec::new();
        for mut stmt in block.stmts.drain(..) {
            if let Stmt::Local(local) = stmt {
                self.process_local(local);
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
                *ident = unique_ident_vec
                    .last()
                    .expect("Scope ident should correspond to unique ident")
                    .clone();
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
        // do not go into a type
    }

    fn visit_member_mut(&mut self, _: &mut Member) {
        // do not go into a member
    }
}

impl Visitor {
    fn process_local(&mut self, mut local: Local) {
        let mut pat = if let Pat::Type(ref mut pat_ty) = &mut local.pat {
            pat_ty.pat.as_mut()
        } else {
            &mut local.pat
        };

        let Pat::Ident(pat_ident) = &mut pat else {
            panic!("Non-ident pattern in scope normalization");
        };

        assert!(pat_ident.subpat.is_none() && pat_ident.by_ref.is_none());

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

        let unique_ident =
            construct_prefixed_ident(&format!("scope_{}_{}", scope_num, shadow_num), &left_ident);
        // add ident to scope
        unique_ident_vec.push(unique_ident.clone());

        // the local does not contain initialization after construct normalization
        assert!(local.init.is_none());

        // assign unique ident to local and push the local to local defs
        pat_ident.ident = unique_ident;
        self.local_defs.push(local);
    }
}
