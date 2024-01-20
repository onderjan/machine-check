use std::{collections::HashMap, hash::Hash, vec};

use syn::{
    visit_mut::{self, VisitMut},
    AngleBracketedGenericArguments, Block, ExprCall, Ident, Item, Pat, PatType, Path,
    PathArguments, Stmt, Type,
};
use syn_path::path;

use crate::{
    support::local::extract_local_ident_with_type,
    util::{
        create_type_path, extract_expr_ident, extract_expr_path, extract_path_ident,
        extract_type_path, path_matches_global_names, single_bit_type,
    },
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

        if self
            .local_ident_types
            .get_mut(&left_ident)
            .expect("Left ident should be in local ident types")
            .is_some()
        {
            // we already have some left type, return
            return;
        }

        let inferred_type = match expr_assign.right.as_ref() {
            syn::Expr::Call(expr_call) => self.infer_call_result_type(expr_call),
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

        // add inferred type
        *self.local_ident_types.get_mut(&left_ident).unwrap() = inferred_type;

        // delegate visit
        visit_mut::visit_expr_assign_mut(self, expr_assign);
    }
}

impl BlockVisitor {
    fn infer_call_result_type(&self, expr_call: &ExprCall) -> Option<Type> {
        // discover the type based on the call function
        let func_path = extract_expr_path(&expr_call.func);
        // --- BITVECTOR INITIALIZATION ---
        if path_matches_global_names(&func_path, &["mck", "concr", "Bitvector", "new"]) {
            // infer bitvector type
            let mut bitvector = path!(::mck::concr::Bitvector);
            bitvector.segments[2].arguments = func_path.segments[2].arguments.clone();
            return Some(create_type_path(bitvector));
        }

        // --- FUNCTIONS THAT ALWAYS RETURN A SINGLE BIT ---
        for (bit_result_trait, bit_result_fn) in BIT_RESULT_TRAIT_FNS {
            if path_matches_global_names(
                &func_path,
                &["mck", "forward", bit_result_trait, bit_result_fn],
            ) {
                return Some(single_bit_type("concr"));
            }
        }

        // --- FUNCTIONS THAT RETAIN ARGUMENT TYPES IN RETURN TYPE ---
        for (bit_result_trait, bit_result_fn) in TYPE_RETAINING_TRAIT_FNS {
            if path_matches_global_names(
                &func_path,
                &["mck", "forward", bit_result_trait, bit_result_fn],
            ) {
                // take the type from first typed argument we find
                for arg in &expr_call.args {
                    let arg_ident = extract_expr_ident(arg);
                    let arg_type = self
                        .local_ident_types
                        .get(&arg_ident)
                        .expect("Call argument should have local ident");
                    if let Some(arg_type) = arg_type {
                        return Some(arg_type.clone());
                    }
                }

                // no joy
                // TODO: error here
                return None;
            }
        }

        // --- FUNCTION THAT CHANGE GENERICS BASED ON TRAIT ---
        for (bit_result_trait, bit_result_fn) in GENERICS_CHANGING_TRAIT_FNS {
            if path_matches_global_names(
                &func_path,
                &["mck", "forward", bit_result_trait, bit_result_fn],
            ) {
                // take the type from first typed argument we find
                for arg in &expr_call.args {
                    let arg_ident = extract_expr_ident(arg);
                    let arg_type = self
                        .local_ident_types
                        .get(&arg_ident)
                        .expect("Call argument should have local ident");
                    if let Some(arg_type) = arg_type {
                        // change the argument generics based on trait generics
                        let mut type_path = extract_type_path(arg_type);
                        type_path.segments[2].arguments = func_path.segments[2].arguments.clone();
                        println!("Extension: {:?}, {:?}", type_path, func_path);

                        return Some(create_type_path(type_path));
                    }
                }

                // no joy
                // TODO: error here
                return None;
            }
        }
        None
    }
}

static BIT_RESULT_TRAIT_FNS: [(&str, &str); 5] = [
    ("TypedEq", "typed_eq"),
    ("TypedCmp", "typed_ult"),
    ("TypedCmp", "typed_slt"),
    ("TypedCmp", "typed_ulte"),
    ("TypedCmp", "typed_slte"),
];

static TYPE_RETAINING_TRAIT_FNS: [(&str, &str); 15] = [
    ("Bitwise", "bit_not"),
    ("Bitwise", "bit_and"),
    ("Bitwise", "bit_or"),
    ("Bitwise", "bit_xor"),
    ("HwArith", "arith_neg"),
    ("HwArith", "add"),
    ("HwArith", "sub"),
    ("HwArith", "mul"),
    ("HwArith", "udiv"),
    ("HwArith", "sdiv"),
    ("HwArith", "urem"),
    ("HwArith", "srem"),
    ("HwShift", "logic_shl"),
    ("HwShift", "logic_shr"),
    ("HwShift", "arith_shr"),
];

static GENERICS_CHANGING_TRAIT_FNS: [(&str, &str); 2] = [("Ext", "uext"), ("Ext", "sext")];
