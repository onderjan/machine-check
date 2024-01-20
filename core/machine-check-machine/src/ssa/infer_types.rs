use std::{collections::HashMap, vec};

use syn::{
    visit_mut::{self, VisitMut},
    Ident, Item, Pat, PatType, Path, Stmt, Type,
};
use syn_path::path;

use crate::{
    support::local::extract_local_ident_with_type,
    util::{create_type_path, extract_expr_ident, extract_expr_path, extract_path_ident},
    MachineError,
};

pub fn infer_types(items: &mut [Item]) -> Result<(), MachineError> {
    let mut visitor = BlockVisitor {
        local_ident_types: HashMap::new(),
        result: Ok(()),
    };
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }

    visitor.result
}

struct BlockVisitor {
    local_ident_types: HashMap<Ident, Option<Type>>,
    result: Result<(), MachineError>,
}
impl VisitMut for BlockVisitor {
    fn visit_impl_item_fn_mut(&mut self, impl_item_fn: &mut syn::ImplItemFn) {
        println!("Visiting item function {:?}", quote::quote!(#impl_item_fn));
        // add local idents
        let mut i = 0;
        while let Stmt::Local(local) = &impl_item_fn.block.stmts[i] {
            // add local ident
            let (local_ident, local_type) = extract_local_ident_with_type(local);
            self.local_ident_types.insert(local_ident, local_type);
            i += 1;
        }

        // perform visits of statements
        visit_mut::visit_impl_item_fn_mut(self, impl_item_fn);

        // merge local types
        let mut i = 0;
        while let Stmt::Local(local) = &mut impl_item_fn.block.stmts[i] {
            match &local.pat {
                Pat::Ident(pat_ident) => {
                    // no type yet
                    println!("Pattern has no type yet: {:?}", pat_ident);
                    let inferred_type = self.local_ident_types.remove(&pat_ident.ident).unwrap();
                    if let Some(inferred_type) = inferred_type {
                        println!("Inferred type: {:?}", inferred_type);
                        // add type
                        local.pat = Pat::Type(PatType {
                            attrs: vec![],
                            pat: Box::new(Pat::Ident(pat_ident.clone())),
                            colon_token: Default::default(),
                            ty: Box::new(inferred_type),
                        })
                    }
                }
                Pat::Type(_) => {
                    // do nothing, we already have the type
                }
                _ => panic!("Unexpected local pattern {:?}", local.pat),
            }
            i += 1;
        }

        // clear local idents
        self.local_ident_types.clear();
    }

    fn visit_expr_assign_mut(&mut self, expr_assign: &mut syn::ExprAssign) {
        let left_ident = extract_expr_ident(&expr_assign.left);
        println!(
            "left ident: {:?}, local ident types: {:?}",
            left_ident, self.local_ident_types
        );

        let inferred_type = match expr_assign.right.as_ref() {
            syn::Expr::Call(expr_call) => {
                // discover the type based on the call function
                let func_path = extract_expr_path(&expr_call.func);
                if func_path.leading_colon.is_none() {
                    panic!("Unexpected non-global call function {:?}", func_path);
                }

                if func_path.segments.len() == 4
                    && &func_path.segments[0].ident.to_string() == "mck"
                    && &func_path.segments[1].ident.to_string() == "concr"
                    && &func_path.segments[2].ident.to_string() == "Bitvector"
                    && &func_path.segments[3].ident.to_string() == "new"
                {
                    // infer bitvector type
                    let mut type_path = path!(::mck::concr::Bitvector);
                    type_path.segments[2].arguments = func_path.segments[2].arguments.clone();
                    Some(create_type_path(type_path))
                } else {
                    None
                }
            }
            syn::Expr::Field(expr_field) => {
                // TODO
                None
            }
            syn::Expr::Path(expr_path) => {
                // TODO
                None
            }
            _ => panic!("Unexpected local assignment expression {:?}", expr_assign),
        };

        if let Some(inferred_type) = inferred_type {
            let current_left_type = self
                .local_ident_types
                .get_mut(&left_ident)
                .expect("Left ident should be in local ident types");
            if let Some(current_left_type) = current_left_type {
                // test for disagreement
                if current_left_type != &inferred_type {
                    panic!("Type inference disagreement");
                }
            } else {
                // add inferred type
                *current_left_type = Some(inferred_type);
            }
        }

        // delegate visit
        visit_mut::visit_expr_assign_mut(self, expr_assign);

        /*println!(
            "Inferring type from expression: {:?}",
            quote::quote!(#expr_assign)
        );*/
    }
}
