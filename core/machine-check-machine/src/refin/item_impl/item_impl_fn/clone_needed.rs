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
        types::find_local_types,
    },
    util::{
        create_assign, create_expr_call, create_expr_ident, create_expr_path, create_expr_tuple,
        extract_expr_ident, path_matches_global_names, ArgType,
    },
};

pub fn clone_needed(impl_item_fn: &mut ImplItemFn) {
    // visit the function to see which idents should be created
    let local_types = find_local_types(impl_item_fn);
    let mut visitor = Visitor {
        local_types: &local_types,
        created_idents: BTreeMap::new(),
    };
    visitor.visit_impl_item_fn_mut(impl_item_fn);

    // add created locals which are mutable and uninit at start
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
        // uninit at start so that the compiler does not complain about not being assigned on all paths
        // (we know that they are initialized on the paths where they matter)
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

    // add attributes to pair the original idents with created clones
    for mut stmt in orig_stmts {
        let Stmt::Local(local) = &mut stmt else {
            impl_item_fn.block.stmts.push(stmt);
            continue;
        };
        let (left_ident, ty) = extract_local_ident_with_type(local);
        let Some(clone_ident) = visitor.created_idents.get(&left_ident) else {
            impl_item_fn.block.stmts.push(stmt);
            continue;
        };
        // has clone, add the attribute
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
        if let Some(ty) = ty {
            if matches!(ty, Type::Reference(_)) {
                // add the information that the type was a reference before
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

            let left_ident = extract_expr_ident(expr_assign.left.as_ref())
                .expect("Assignment expression left side should be ident");

            // do not clone if it is a PhiArg, it would be useless
            let Some(left_ident_type) = self.local_types.get(left_ident) else {
                // expression left not in local types
                // this should be a compile error later, so ignore now
                continue;
            };

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

            // construct the ident and add the clone statement
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
        }
        block.stmts = processed_stmts;
    }
}
