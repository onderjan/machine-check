use std::collections::HashMap;

use syn::{
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    ExprCall, Ident, ItemStruct, Path, PathArguments, PathSegment, Type,
};

use crate::{
    util::{extract_expr_path_mut, path_matches_global_names, path_starts_with_global_names},
    MachineError,
};

pub struct LocalVisitor<'a> {
    pub local_ident_types: HashMap<Ident, Type>,
    pub structs: &'a HashMap<Path, ItemStruct>,
    pub result: Result<(), MachineError>,
}

impl VisitMut for LocalVisitor<'_> {
    fn visit_expr_call_mut(&mut self, expr_call: &mut ExprCall) {
        println!("Visit expr call: {}", quote::quote!(#expr_call));
        let func_path =
            extract_expr_path_mut(&mut expr_call.func).expect("Call function should be path");

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

        // TODO: div, rem depending on Signed/Unsigned

        // TODO: HW shift, extension

        // delegate
        visit_mut::visit_expr_call_mut(self, expr_call);
    }

    fn visit_path_mut(&mut self, path: &mut Path) {
        println!("Visit path: {}, {:?}", quote::quote!(#path), path);
        if path_starts_with_global_names(path, &["machine_check", "Bitvector"])
            || path_starts_with_global_names(path, &["machine_check", "Unsigned"])
            || path_starts_with_global_names(path, &["machine_check", "Signed"])
        {
            println!("Matches bitvector!");
            let first_segment_span = path.segments[0].span();
            path.segments[0].ident = Ident::new("mck", first_segment_span);
            path.segments.insert(
                1,
                PathSegment {
                    ident: Ident::new("concr", first_segment_span),
                    arguments: PathArguments::None,
                },
            );
            path.segments[2].ident = Ident::new("Bitvector", path.segments[2].ident.span());
        }
        if path_starts_with_global_names(path, &["machine_check", "BitvectorArray"]) {
            println!("Matches array!");
            let first_segment_span = path.segments[0].span();
            path.segments[0].ident = Ident::new("mck", first_segment_span);
            path.segments.insert(
                1,
                PathSegment {
                    ident: Ident::new("concr", first_segment_span),
                    arguments: PathArguments::None,
                },
            );
            path.segments[2].ident = Ident::new("Array", path.segments[2].ident.span());
        }

        // delegate
        visit_mut::visit_path_mut(self, path);
    }
}
