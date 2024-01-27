use syn::{
    visit_mut::{self, VisitMut},
    Attribute, Expr, ExprPath, Ident, ImplItemFn, MetaNameValue, Stmt,
};
use syn_path::path;

use crate::{
    support::{local::construct_prefixed_ident, local_types::find_local_types},
    util::{
        create_assign, create_expr_call, create_expr_ident, create_expr_path, create_let_bare,
        extract_expr_ident, path_matches_global_names, ArgType,
    },
};

pub fn clone_needed(impl_item_fn: &mut ImplItemFn) {
    let local_types = find_local_types(impl_item_fn);
    let mut visitor = Visitor {
        created_idents: Vec::new(),
    };
    visitor.visit_impl_item_fn_mut(impl_item_fn);

    // add created locals
    let mut stmts: Vec<Stmt> = Vec::new();
    for (created_ident, orig_ident) in visitor.created_idents {
        let ty = local_types
            .get(&orig_ident)
            .expect("Created local original should be in local types");
        stmts.push(create_let_bare(created_ident, Some(ty.clone())));
    }
    stmts.append(&mut impl_item_fn.block.stmts);
    impl_item_fn.block.stmts = stmts;
}

struct Visitor {
    created_idents: Vec<(Ident, Ident)>,
}

impl VisitMut for Visitor {
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
            let Expr::Call(expr_call) = expr_assign.right.as_mut() else {
                processed_stmts.push(stmt);
                continue;
            };

            let Expr::Path(ExprPath { path, .. }) = expr_call.func.as_ref() else {
                panic!("Unexpected non-path call function");
            };
            if path_matches_global_names(path, &["mck", "forward", "ReadWrite", "write"]) {
                // clone the first argument
                let first_arg_ident =
                    extract_expr_ident(&expr_call.args[0]).expect("Write argument should be ident");

                let clone_ident = construct_prefixed_ident("clone", first_arg_ident);
                self.created_idents
                    .push((clone_ident.clone(), first_arg_ident.clone()));
                let clone_call = create_expr_call(
                    create_expr_path(path!(::std::clone::Clone::clone)),
                    vec![(
                        ArgType::Reference,
                        create_expr_ident(first_arg_ident.clone()),
                    )],
                );
                let clone_assign = create_assign(clone_ident.clone(), clone_call, true);
                processed_stmts.push(clone_assign);
                // add attribute for clone to the original call
                expr_call.attrs.push(Attribute {
                    pound_token: Default::default(),
                    style: syn::AttrStyle::Outer,
                    bracket_token: Default::default(),
                    meta: syn::Meta::NameValue(MetaNameValue {
                        path: path!(::mck::attr::refin_clone),
                        eq_token: Default::default(),
                        value: create_expr_ident(clone_ident),
                    }),
                });
            }

            processed_stmts.push(stmt);
        }
        block.stmts = processed_stmts;
    }
}
