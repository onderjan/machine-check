use std::collections::HashMap;

use syn::{
    visit_mut::{self, VisitMut},
    ExprCall, ExprField, Ident, ItemStruct, Member, Path, Type,
};
use syn_path::path;

use crate::{
    util::{
        create_path_with_last_generic_type, create_type_path, extract_expr_ident,
        extract_expr_path, extract_last_generic_type, extract_path_ident, extract_type_path,
        path_matches_global_names, single_bit_type,
    },
    MachineError,
};

use super::{
    fn_properties::{BIT_RESULT_TRAIT_FNS, GENERICS_CHANGING_TRAIT_FNS, TYPE_RETAINING_TRAIT_FNS},
    type_properties::is_type_standard_inferred,
};

pub struct LocalVisitor<'a> {
    pub local_ident_types: HashMap<Ident, Option<Type>>,
    pub structs: &'a HashMap<Path, ItemStruct>,
    pub result: Result<(), MachineError>,
}

impl VisitMut for LocalVisitor<'_> {
    fn visit_expr_assign_mut(&mut self, expr_assign: &mut syn::ExprAssign) {
        let left_ident =
            extract_expr_ident(&expr_assign.left).expect("Left side of assignment should be ident");

        if let Some(ty) = self
            .local_ident_types
            .get_mut(left_ident)
            .expect("Left ident should be in local ident types")
        {
            if is_type_standard_inferred(ty) {
                // we already have determined left type, return
                return;
            }
        }

        let inferred_type = match expr_assign.right.as_ref() {
            syn::Expr::Call(right_call) => self.infer_call_result_type(right_call),
            syn::Expr::Field(right_field) => self.infer_field_result_type(right_field),
            syn::Expr::Path(right_path) => {
                // infer from the right identifier
                let right_ident = extract_path_ident(&right_path.path)
                    .expect("Right side of assignment should be ident");
                let right_type = self
                    .local_ident_types
                    .get(right_ident)
                    .expect("Right ident should be in ident types");
                right_type.clone()
            }
            _ => panic!("Unexpected local assignment expression {:?}", expr_assign),
        };

        // add inferred type
        if let Some(inferred_type) = inferred_type {
            *self.local_ident_types.get_mut(left_ident).unwrap() = Some(inferred_type);
        }

        // delegate visit
        visit_mut::visit_expr_assign_mut(self, expr_assign);
    }
}

impl LocalVisitor<'_> {
    fn infer_field_result_type(&self, expr_field: &ExprField) -> Option<Type> {
        // get type of member from structs
        let base_ident =
            extract_expr_ident(expr_field.base.as_ref()).expect("Field base should be an ident");
        let base_type = self
            .local_ident_types
            .get(base_ident)
            .expect("Base ident should be in ident types")
            .as_ref();
        let Some(mut base_type) = base_type else {
            return None;
        };
        // ignore references
        while let Type::Reference(ref_type) = base_type {
            base_type = ref_type.elem.as_ref();
        }

        let base_type_path = extract_type_path(base_type);
        let base_struct = self.structs.get(&base_type_path);
        let Some(base_struct) = base_struct else {
            return None;
        };
        match &base_struct.fields {
            syn::Fields::Named(fields) => {
                // match ident
                let Member::Named(member_ident) = &expr_field.member else {
                    return None;
                };
                let Some(field) = fields.named.iter().find(|field| {
                    let field_ident = field.ident.as_ref().unwrap();
                    field_ident == member_ident
                }) else {
                    return None;
                };
                Some(field.ty.clone())
            }
            syn::Fields::Unnamed(fields) => {
                let Member::Unnamed(member_index) = &expr_field.member else {
                    return None;
                };
                let Some(field) = fields.unnamed.iter().nth(member_index.index as usize) else {
                    return None;
                };
                Some(field.ty.clone())
            }
            syn::Fields::Unit => None,
        }
    }

    fn infer_call_result_type(&self, expr_call: &ExprCall) -> Option<Type> {
        // discover the type based on the call function
        let func_path = extract_expr_path(&expr_call.func).expect("Call function should be path");
        // --- BITVECTOR INITIALIZATION ---
        if path_matches_global_names(func_path, &["mck", "concr", "Bitvector", "new"]) {
            // infer bitvector type
            let mut bitvector = path!(::mck::concr::Bitvector);
            bitvector.segments[2].arguments = func_path.segments[2].arguments.clone();
            return Some(create_type_path(bitvector));
        }
        if path_matches_global_names(func_path, &["mck", "concr", "Array", "new_filled"]) {
            // infer array type
            let mut array = path!(::mck::concr::Array);
            array.segments[2].arguments = func_path.segments[2].arguments.clone();
            return Some(create_type_path(array));
        }

        if path_matches_global_names(func_path, &["mck", "forward", "ReadWrite", "write"]) {
            // infer from first argument
            let arg = &expr_call.args[0];
            // take the type from first typed argument we find
            let arg_ident = extract_expr_ident(arg).expect("Call argument should be ident");
            let arg_type = self
                .local_ident_types
                .get(arg_ident)
                .expect("Call argument should have local ident");
            if let Some(arg_type) = arg_type {
                return Some(arg_type.clone());
            }

        }

        // --- FUNCTIONS THAT ALWAYS RETURN A SINGLE BIT ---
        for (bit_result_trait, bit_result_fn) in BIT_RESULT_TRAIT_FNS {
            if path_matches_global_names(
                func_path,
                &["mck", "forward", bit_result_trait, bit_result_fn],
            ) {
                return Some(single_bit_type("concr"));
            }
        }

        // --- FUNCTIONS THAT RETAIN ARGUMENT TYPES IN RETURN TYPE ---
        for (bit_result_trait, bit_result_fn) in TYPE_RETAINING_TRAIT_FNS {
            if path_matches_global_names(
                func_path,
                &["mck", "forward", bit_result_trait, bit_result_fn],
            ) {
                // take the type from first typed argument we find
                for arg in &expr_call.args {
                    let arg_ident = extract_expr_ident(arg).expect("Call argument should be ident");
                    let arg_type = self
                        .local_ident_types
                        .get(arg_ident)
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
                func_path,
                &["mck", "forward", bit_result_trait, bit_result_fn],
            ) {
                // take the type from first typed argument we find
                for arg in &expr_call.args {
                    let arg_ident = extract_expr_ident(arg).expect("Call argument should be ident");
                    let arg_type = self
                        .local_ident_types
                        .get(arg_ident)
                        .expect("Call argument should have local ident");
                    if let Some(arg_type) = arg_type {
                        // change the argument generics based on trait generics
                        let mut type_path = extract_type_path(arg_type);
                        type_path.segments[2].arguments = func_path.segments[2].arguments.clone();

                        return Some(create_type_path(type_path));
                    }
                }

                // no joy
                // TODO: error here
                return None;
            }
        }

        if path_matches_global_names(func_path, &["mck", "forward", "PhiArg", "Taken"]) {
            assert!(expr_call.args.len() == 1);
            let arg_ident =
                extract_expr_ident(&expr_call.args[0]).expect("Call argument should be ident");
            let arg_type = self
                .local_ident_types
                .get(arg_ident)
                .expect("Call argument should have local ident");
            if let Some(arg_type) = arg_type {
                return Some(create_type_path(create_path_with_last_generic_type(
                    path!(::mck::forward::PhiArg),
                    arg_type.clone(),
                )));
            }
        }

        if path_matches_global_names(func_path, &["mck", "forward", "PhiArg", "phi"]) {
            assert!(expr_call.args.len() == 2);
            for arg in &expr_call.args {
                let arg_ident = extract_expr_ident(arg).expect("Call argument should be ident");
                let arg_type = self
                    .local_ident_types
                    .get(arg_ident)
                    .expect("Call argument should have local ident");
                if let Some(arg_type) = arg_type {
                    // extract
                    if let Some(ty) = extract_last_generic_type(extract_type_path(arg_type)) {
                        return Some(ty);
                    }
                }
            }
        }

        None
    }
}