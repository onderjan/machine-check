use core::panic;
use std::collections::HashMap;

use syn::{
    punctuated::Punctuated,
    visit_mut::{self, VisitMut},
    AngleBracketedGenericArguments, ExprCall, ExprField, ExprIndex, ExprPath, ExprReference, Ident,
    ItemStruct, Member, Path, PathArguments, Type, TypeReference,
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
            syn::Expr::Path(right_path) => self.infer_path_result_type(right_path),
            syn::Expr::Call(right_call) => self.infer_call_result_type(right_call),
            syn::Expr::Field(right_field) => self.infer_field_result_type(right_field),
            syn::Expr::Index(right_index) => self.infer_index_result_type(right_index),
            syn::Expr::Reference(right_reference) => {
                self.infer_reference_result_type(right_reference)
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
            .expect("Field base ident should be in ident types")
            .as_ref();
        let Some(mut base_type) = base_type else {
            return None;
        };
        // dereference first
        while let Type::Reference(ref_type) = base_type {
            base_type = ref_type.elem.as_ref();
        }

        let Some(base_type_path) = extract_type_path(base_type) else {
            panic!("Unexpected base type: {:?}", base_type);
        };

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

    fn infer_index_result_type(&self, expr_index: &ExprIndex) -> Option<Type> {
        println!(
            "Inferring index result type: {}",
            quote::quote!(#expr_index)
        );
        // infer type of base expression
        let base_ident = extract_expr_ident(&expr_index.expr).expect("Index base should be ident");
        let base_type = self
            .local_ident_types
            .get(base_ident)
            .expect("Index base ident should be in ident types")
            .as_ref();
        let Some(mut base_type) = base_type else {
            return None;
        };

        // dereference first
        while let Type::Reference(ref_type) = base_type {
            base_type = ref_type.elem.as_ref();
        }

        let Some(base_type_path) = extract_type_path(base_type) else {
            panic!("Unexpected index base type: {:?}", base_type);
        };

        println!("Base type path: {}", quote::quote!(#base_type_path));

        if path_matches_global_names(&base_type_path, &["machine_check", "BitvectorArray"]) {
            // infer bitvector type with element length, which is the second generic argument
            let mut bitvector_path = path!(::machine_check::Bitvector);
            let PathArguments::AngleBracketed(array_angle_bracketed) =
                &base_type_path.segments[1].arguments
            else {
                panic!("Expected generic arguments to array");
            };
            if array_angle_bracketed.args.len() != 2 {
                panic!("Expected exactly two generic arguments to array");
            }
            let mut bitvector_angle_bracketed = array_angle_bracketed.clone();
            bitvector_angle_bracketed.args =
                Punctuated::from_iter([bitvector_angle_bracketed.args.pop().unwrap()]);
            bitvector_path.segments[1].arguments =
                PathArguments::AngleBracketed(bitvector_angle_bracketed);
            Some(create_type_path(bitvector_path))
        } else {
            None
        }
    }

    fn infer_call_result_type(&self, expr_call: &ExprCall) -> Option<Type> {
        // discover the type based on the call function
        let func_path = extract_expr_path(&expr_call.func).expect("Call function should be path");
        // --- BITVECTOR INITIALIZATION ---
        if path_matches_global_names(func_path, &["machine_check", "Bitvector", "new"]) {
            // infer bitvector type
            let mut bitvector = path!(::machine_check::Bitvector);
            bitvector.segments[1].arguments = func_path.segments[1].arguments.clone();
            return Some(create_type_path(bitvector));
        }
        if path_matches_global_names(
            func_path,
            &["machine_check", "BitvectorArray", "new_filled"],
        ) {
            // infer array type
            let mut array = path!(::machine_check::BitvectorArray);
            array.segments[1].arguments = func_path.segments[1].arguments.clone();
            return Some(create_type_path(array));
        }

        /*if path_matches_global_names(func_path, &["mck", "forward", "ReadWrite", "write"]) {
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

        if path_matches_global_names(func_path, &["mck", "forward", "ReadWrite", "read"]) {
            // infer from first argument which should be a reference to the array
            let arg = &expr_call.args[0];
            // take the type from first typed argument we find
            let arg_ident = extract_expr_ident(arg).expect("Call argument should be ident");
            let arg_type = self
                .local_ident_types
                .get(arg_ident)
                .expect("Call argument should have local ident");
            if let Some(arg_type) = arg_type {
                // the argument type is a reference to the array, construct the bitvector type
                let Type::Reference(type_reference) = arg_type else {
                    panic!("Expected first argument of array read to be a reference");
                };
                let array_type = type_reference.elem.as_ref();
                let Some(array_path) = extract_type_path(array_type) else {
                    panic!("Expected first argument of array read to be a reference to path type");
                };
                if !path_matches_global_names(&array_path, &["mck", "concr", "Array"]) {
                    panic!("Expected first argument of array read to be a reference to Array");
                }
                let PathArguments::AngleBracketed(generics) = &array_path.segments[2].arguments
                else {
                    panic!("Expected first argument of array read to have generic arguments");
                };
                if generics.args.len() != 2 {
                    panic!("Expected first argument of array read to have exactly two generic arguments");
                }
                // element length is the second argument
                let mut result_type_path = path!(::mck::concr::Bitvector);
                result_type_path.segments[2].arguments =
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: Default::default(),
                        lt_token: Default::default(),
                        args: Punctuated::from_iter(vec![generics.args[1].clone()]),
                        gt_token: Default::default(),
                    });

                return Some(create_type_path(result_type_path));
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
                        if let Some(mut type_path) = extract_type_path(arg_type) {
                            // change the argument generics based on trait generics
                            type_path.segments[2].arguments =
                                func_path.segments[2].arguments.clone();

                            return Some(create_type_path(type_path));
                        }
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
                    // extract, never a reference
                    if let Some(arg_path) = extract_type_path(arg_type) {
                        if let Some(ty) = extract_last_generic_type(arg_path) {
                            return Some(ty);
                        }
                    }
                }
            }
        }*/

        None
    }

    fn infer_path_result_type(&self, expr_path: &ExprPath) -> Option<Type> {
        // infer from the identifier
        let right_ident =
            extract_path_ident(&expr_path.path).expect("Right side of assignment should be ident");
        let right_type = self
            .local_ident_types
            .get(right_ident)
            .expect("Right ident should be in ident types");
        right_type.clone()
    }

    fn infer_reference_result_type(&self, expr_reference: &ExprReference) -> Option<Type> {
        // infer type from the identifier first
        let expr_ident = extract_expr_ident(&expr_reference.expr)
            .expect("Right side of assignment should be ident");
        let expr_type = self
            .local_ident_types
            .get(expr_ident)
            .expect("Right ident should be in ident types")
            .clone();
        expr_type.map(|expr_type|
            // resolve to reference
            Type::Reference(TypeReference {
                and_token: Default::default(),
                lifetime: None,
                mutability: None,
                elem: Box::new(expr_type.clone()),
        }))
    }
}
