use std::collections::{BTreeMap, HashMap};

use syn::{
    visit_mut::{self, VisitMut},
    Attribute, Expr, Ident, ImplItemFn, Local, LocalInit, MetaNameValue, Pat, PatIdent, PatType,
    Stmt, Token, Type,
};
use syn_path::path;

use crate::{
    support::{
        local::{construct_prefixed_ident, extract_local_ident_with_type},
        local_types::find_local_types,
    },
    util::{
        create_assign, create_expr_call, create_expr_ident, create_expr_path, create_expr_tuple,
        extract_expr_ident, path_matches_global_names, ArgType,
    },
};

pub fn clone_needed(impl_item_fn: &mut ImplItemFn) {
    let local_types = find_local_types(impl_item_fn);
    let mut visitor = Visitor {
        local_types: &local_types,
        created_idents: BTreeMap::new(),
    };
    visitor.visit_impl_item_fn_mut(impl_item_fn);

    // add created locals
    let mut clone_local_stmts: Vec<Stmt> = Vec::new();
    for (orig_ident, created_ident) in visitor.created_idents.iter() {
        let ty = local_types
            .get(orig_ident)
            .expect("Created local original should be in local types");
        let mut clone_ty = ty.clone();
        if let Type::Reference(ref_ty) = clone_ty {
            clone_ty = ref_ty.elem.as_ref().clone();
        }

        let pat = Pat::Type(PatType {
            attrs: vec![],
            colon_token: Token![:](orig_ident.span()),
            pat: Box::new(Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: Some(Token![mut](orig_ident.span())),
                ident: created_ident.clone(),
                subpat: None,
            })),
            ty: Box::new(clone_ty),
        });
        // default uninit
        let default_call_expr =
            create_expr_call(create_expr_path(path!(::mck::abstr::Phi::uninit)), vec![]);
        let init = LocalInit {
            eq_token: Token![=](orig_ident.span()),
            expr: Box::new(default_call_expr),
            diverge: None,
        };
        clone_local_stmts.push(Stmt::Local(Local {
            attrs: vec![],
            let_token: Token![let](orig_ident.span()),
            pat,
            init: Some(init),
            semi_token: Token![;](orig_ident.span()),
        }));
    }

    let orig_stmts = Vec::from_iter(impl_item_fn.block.stmts.drain(..));

    impl_item_fn.block.stmts.extend(clone_local_stmts);

    for mut stmt in orig_stmts {
        if let Stmt::Local(local) = &mut stmt {
            let (left_ident, ty) = extract_local_ident_with_type(local);
            if let Some(clone_ident) = visitor.created_idents.get(&left_ident) {
                if let Some(ty) = ty {
                    if matches!(ty, Type::Reference(_)) {
                        local.attrs.push(Attribute {
                            pound_token: Default::default(),
                            style: syn::AttrStyle::Outer,
                            bracket_token: Default::default(),
                            meta: syn::Meta::NameValue(MetaNameValue {
                                path: path!(::mck::attr::reference_clone),
                                eq_token: Default::default(),
                                value: create_expr_tuple(vec![]),
                            }),
                        });
                    }
                }
                local.attrs.push(Attribute {
                    pound_token: Default::default(),
                    style: syn::AttrStyle::Outer,
                    bracket_token: Default::default(),
                    meta: syn::Meta::NameValue(MetaNameValue {
                        path: path!(::mck::attr::refin_clone),
                        eq_token: Default::default(),
                        value: create_expr_ident(clone_ident.clone()),
                    }),
                });
            }
        }
        impl_item_fn.block.stmts.push(stmt);
    }
}

struct Visitor<'a> {
    local_types: &'a HashMap<Ident, syn::Type>,
    created_idents: BTreeMap<Ident, Ident>,
}

impl VisitMut for Visitor<'_> {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        // delegate first so we do not forget
        visit_mut::visit_block_mut(self, block);

        // process statements
        let mut processed_stmts = Vec::new();
        for mut stmt in block.stmts.drain(..) {
            let Stmt::Expr(Expr::Assign(expr_assign), _) = &mut stmt else {
                processed_stmts.push(stmt);
                continue;
            };
            /*let Expr::Call(expr_call) = expr_assign.right.as_mut() else {
                processed_stmts.push(stmt);
                continue;
            };*/

            /*let Expr::Path(ExprPath { path, .. }) = expr_call.func.as_ref() else {
                panic!("Unexpected non-path call function");
            };
            if path_matches_global_names(path, &["mck", "forward", "ReadWrite", "write"]) {
                // clone the first argument
                let first_arg_ident =
                    extract_expr_ident(&expr_call.args[0]).expect("Write argument should be ident");*/

            let left_ident = extract_expr_ident(expr_assign.left.as_ref())
                .expect("Assignment expression left side should be ident");

            // do not clone if it is a PhiArg, it would be useless
            let left_ident_type = self
                .local_types
                .get(left_ident)
                .expect("Assignment expression left should be in local types");

            if let Type::Path(type_path) = left_ident_type {
                if path_matches_global_names(&type_path.path, &["mck", "forward", "PhiArg"]) {
                    // skip

                    processed_stmts.push(stmt);
                    continue;
                }
            }

            let arg_type = if matches!(left_ident_type, Type::Reference(_)) {
                ArgType::Normal
            } else {
                ArgType::Reference
            };

            let clone_ident = construct_prefixed_ident("clone", left_ident);
            self.created_idents
                .insert(left_ident.clone(), clone_ident.clone());
            let clone_call = create_expr_call(
                create_expr_path(path!(::std::clone::Clone::clone)),
                vec![(arg_type, create_expr_ident(left_ident.clone()))],
            );
            let clone_assign = create_assign(clone_ident.clone(), clone_call, true);

            processed_stmts.push(stmt);
            processed_stmts.push(clone_assign);

            // add attribute for clone to the original call
            /*expr_call.attrs.push(Attribute {
                pound_token: Default::default(),
                style: syn::AttrStyle::Outer,
                bracket_token: Default::default(),
                meta: syn::Meta::NameValue(MetaNameValue {
                    path: path!(::mck::attr::refin_clone),
                    eq_token: Default::default(),
                    value: create_expr_ident(clone_ident),
                }),
            });*/

            //}
        }
        block.stmts = processed_stmts;
    }
}
