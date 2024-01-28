use std::collections::HashMap;

use syn::{
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Expr, ExprInfer, Ident, ItemStruct, Path, PathSegment, Type,
};

use crate::{
    util::{extract_expr_ident, extract_expr_path_mut, path_matches_global_names},
    MachineError,
};

pub struct LocalVisitor<'a> {
    pub local_ident_types: HashMap<Ident, Type>,
    pub structs: &'a HashMap<Path, ItemStruct>,
    pub result: Result<(), MachineError>,
}

impl VisitMut for LocalVisitor<'_> {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        let Expr::Call(expr_call) = expr else {
            // just delegate and return
            visit_mut::visit_expr_mut(self, expr);
            return;
        };

        let func_path =
            extract_expr_path_mut(&mut expr_call.func).expect("Call function should be path");

        // --- Into ---
        if path_matches_global_names(func_path, &["std", "convert", "Into", "into"]) {
            // is no-op for our converted types
            // change to no-op
            // TODO: make sure it really works on our bitvector type
            if expr_call.args.len() != 1 {
                panic!("Into should have exactly one argument");
            }
            let mut inner_expr = Expr::Infer(ExprInfer {
                attrs: vec![],
                underscore_token: Default::default(),
            });
            std::mem::swap(&mut inner_expr, &mut expr_call.args[0]);
            *expr = inner_expr;
            // delegate and return
            visit_mut::visit_expr_mut(self, expr);
            return;
        }

        // --- Bitwise ---
        if path_matches_global_names(func_path, &["std", "ops", "Not", "not"]) {
            func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
            func_path.segments[1].ident = Ident::new("forward", func_path.segments[1].span());
            func_path.segments[2].ident = Ident::new("Bitwise", func_path.segments[2].span());
            func_path.segments[3].ident = Ident::new("bit_not", func_path.segments[3].span());
        }
        if path_matches_global_names(func_path, &["std", "ops", "BitAnd", "bitand"]) {
            func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
            func_path.segments[1].ident = Ident::new("forward", func_path.segments[1].span());
            func_path.segments[2].ident = Ident::new("Bitwise", func_path.segments[2].span());
            func_path.segments[3].ident = Ident::new("bit_and", func_path.segments[3].span());
        }
        if path_matches_global_names(func_path, &["std", "ops", "BitOr", "bitor"]) {
            func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
            func_path.segments[1].ident = Ident::new("forward", func_path.segments[1].span());
            func_path.segments[2].ident = Ident::new("Bitwise", func_path.segments[2].span());
            func_path.segments[3].ident = Ident::new("bit_or", func_path.segments[3].span());
        }
        if path_matches_global_names(func_path, &["std", "ops", "BitXor", "bitxor"]) {
            func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
            func_path.segments[1].ident = Ident::new("forward", func_path.segments[1].span());
            func_path.segments[2].ident = Ident::new("Bitwise", func_path.segments[2].span());
            func_path.segments[3].ident = Ident::new("bit_xor", func_path.segments[3].span());
        }

        // --- HwArith ---
        // TODO: ensure they are Signed/Unsigned
        if path_matches_global_names(func_path, &["std", "ops", "Neg", "neg"]) {
            func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
            func_path.segments[1].ident = Ident::new("forward", func_path.segments[1].span());
            func_path.segments[2].ident = Ident::new("HwArith", func_path.segments[2].span());
            func_path.segments[3].ident = Ident::new("arith_neg", func_path.segments[3].span());
        }

        if path_matches_global_names(func_path, &["std", "ops", "Add", "add"])
            || path_matches_global_names(func_path, &["std", "ops", "Sub", "sub"])
            || path_matches_global_names(func_path, &["std", "ops", "Mul", "mul"])
        {
            func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
            func_path.segments[1].ident = Ident::new("forward", func_path.segments[1].span());
            func_path.segments[2].ident = Ident::new("HwArith", func_path.segments[2].span());
            // leave the last segment as-is
        }

        // --- Eq ---
        if path_matches_global_names(func_path, &["std", "cmp", "PartialEq", "eq"])
            || path_matches_global_names(func_path, &["std", "cmp", "PartialEq", "ne"])
        {
            func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
            func_path.segments[1].ident = Ident::new("forward", func_path.segments[1].span());
            func_path.segments[2].ident = Ident::new("TypedEq", func_path.segments[2].span());
            // leave the last segment as-is
        }

        // TODO: div, rem depending on Signed/Unsigned

        // TODO: HW shift, extension

        // --- Shr ---
        if path_matches_global_names(func_path, &["std", "ops", "Shr", "shr"]) {
            // TODO: in Rust, type inference depends on whether Shr is an operation or a call

            func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
            func_path.segments[1].ident = Ident::new("forward", func_path.segments[1].span());
            func_path.segments[2].ident = Ident::new("HwShift", func_path.segments[2].span());

            // need to know type signedness
            if expr_call.args.len() != 2 {
                panic!("Shr should have exactly two arguments");
            }

            let Some(is_signed) = self.is_expr_signed(&expr_call.args[0]) else {
                panic!("Could not determine shr signedness");
            };

            let func_name = if is_signed { "arith_shr" } else { "logic_shr" };
            func_path.segments[3].ident = Ident::new(func_name, func_path.segments[3].span());
        }

        // --- Ext ---
        if path_matches_global_names(func_path, &["machine_check", "Ext", "ext"]) {
            // need to know type signedness
            if expr_call.args.len() != 1 {
                panic!("Ext should have exactly one argument");
            }
            func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
            func_path.segments.insert(
                1,
                PathSegment {
                    ident: Ident::new("forward", func_path.segments[0].span()),
                    arguments: syn::PathArguments::None,
                },
            );

            let Some(is_signed) = self.is_expr_signed(&expr_call.args[0]) else {
                panic!("Could not determine ext signedness");
            };

            let func_name = if is_signed { "uext" } else { "sext" };
            func_path.segments[3].ident = Ident::new(func_name, func_path.segments[3].span());
        }

        // delegate
        visit_mut::visit_expr_mut(self, expr);
    }
}

impl LocalVisitor<'_> {
    fn is_expr_signed(&self, expr: &Expr) -> Option<bool> {
        let right_ident = extract_expr_ident(expr).expect("Expr should be ident");
        let right_ty = self
            .local_ident_types
            .get(right_ident)
            .expect("Type should be in local ident types");
        let Type::Path(right_ty_path) = right_ty else {
            panic!("Local ident type should be a path");
        };
        if path_matches_global_names(&right_ty_path.path, &["machine_check", "Unsigned"]) {
            Some(false)
        } else if path_matches_global_names(&right_ty_path.path, &["machine_check", "Signed"]) {
            Some(true)
        } else {
            None
        }
    }
}
