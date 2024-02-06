use core::panic;
use std::collections::HashMap;

use syn::{
    punctuated::Punctuated,
    visit_mut::{self, VisitMut},
    AngleBracketedGenericArguments, Expr, ExprCall, ExprField, ExprPath, ExprReference, ExprStruct,
    GenericArgument, Ident, ItemStruct, Member, Path, PathArguments, Type, TypeReference,
};
use syn_path::path;

use crate::{
    util::{
        boolean_type, create_type_path, extract_expr_ident, extract_expr_path, extract_path_ident,
        extract_type_path, path_matches_global_names,
    },
    MachineError,
};

use super::{
    fn_properties::{STD_CMP_FNS, STD_OPS_FNS},
    type_properties::is_type_standard_inferred,
};

pub struct LocalVisitor<'a> {
    pub local_ident_types: HashMap<Ident, Option<Type>>,
    pub structs: &'a HashMap<Path, ItemStruct>,
    pub result: Result<(), MachineError>,
    pub inferred_something: bool,
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
                /*println!(
                    "Type of {} is already determined: {}",
                    left_ident,
                    quote::quote!(#ty)
                );*/
                return;
            }
        }

        let inferred_type = match expr_assign.right.as_ref() {
            syn::Expr::Path(right_path) => self.infer_path_result_type(right_path),
            syn::Expr::Call(right_call) => self.infer_call_result_type(right_call),
            syn::Expr::Field(right_field) => self.infer_field_result_type(right_field),
            syn::Expr::Reference(right_reference) => {
                /*println!(
                    "Inferring ident {} from reference {}",
                    left_ident,
                    quote::quote!(#right_reference)
                );*/
                self.infer_reference_result_type(right_reference)
            }
            syn::Expr::Struct(right_struct) => Some(create_type_path(right_struct.path.clone())),
            _ => panic!(
                "Unexpected local assignment expression {} ({:?})",
                quote::quote!(#expr_assign),
                expr_assign
            ),
        };

        // add inferred type
        if let Some(inferred_type) = inferred_type {
            let mut_ty = self.local_ident_types.get_mut(left_ident).unwrap();
            if mut_ty.is_none() {
                /*println!(
                    "Inferred ident {} type {}",
                    left_ident,
                    quote::quote!(#inferred_type)
                );*/
                *self.local_ident_types.get_mut(left_ident).unwrap() = Some(inferred_type);
                self.inferred_something = true;
            }
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

    fn infer_call_result_type(&self, expr_call: &ExprCall) -> Option<Type> {
        /*println!(
            "Inferring call result type for {}",
            quote::quote!(#expr_call)
        );*/

        // discover the type based on the call function
        let func_path = extract_expr_path(&expr_call.func).expect("Call function should be path");
        // --- BITVECTOR INITIALIZATION ---
        if path_matches_global_names(func_path, &["machine_check", "Bitvector", "new"])
            || path_matches_global_names(func_path, &["machine_check", "Unsigned", "new"])
            || path_matches_global_names(func_path, &["machine_check", "Signed", "new"])
        {
            // infer bitvector type
            let mut bitvector = func_path.clone();
            bitvector.segments.pop();
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

        // --- INTO ---

        if path_matches_global_names(func_path, &["std", "convert", "Into", "into"]) {
            // the argument can be given
            let PathArguments::AngleBracketed(angle_bracketed) = &func_path.segments[2].arguments
            else {
                return None;
            };
            if angle_bracketed.args.len() != 1 {
                panic!("Into should have exactly one generic argument");
            }
            let GenericArgument::Type(ty) = &angle_bracketed.args[0] else {
                panic!("Into should have type generic argument");
            };

            return Some(ty.clone());
        }

        if path_matches_global_names(func_path, &["std", "clone", "Clone", "clone"]) {
            // infer from first argument which should be a reference
            let arg = &expr_call.args[0];
            // take the type from first typed argument we find
            let arg_ident =
                extract_expr_ident(arg).expect("Call argument should be reference to ident");
            let arg_type = self
                .local_ident_types
                .get(arg_ident)
                .expect("Call argument should have local ident");
            if let Some(arg_type) = arg_type {
                // the argument type is a reference, dereference it
                let Type::Reference(type_reference) = arg_type else {
                    panic!("Expected first argument of array read to be a reference");
                };
                return Some(type_reference.elem.as_ref().clone());
            }
        }

        if path_matches_global_names(func_path, &["mck", "forward", "ReadWrite", "write"]) {
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

                return Some(array_type.clone());
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
                if !path_matches_global_names(&array_path, &["machine_check", "BitvectorArray"]) {
                    panic!("Expected first argument of array read to be a reference to array");
                }
                let PathArguments::AngleBracketed(generics) = &array_path.segments[1].arguments
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

        // --- FUNCTIONS THAT RETAIN ARGUMENT TYPES IN RETURN TYPE ---
        for (bit_result_trait, bit_result_fn) in STD_OPS_FNS {
            if path_matches_global_names(
                func_path,
                &["std", "ops", bit_result_trait, bit_result_fn],
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

        // TODO: add extensions and conditions
        // --- FUNCTIONS THAT RETURN BOOLEAN ---
        for (bit_result_trait, bit_result_fn) in STD_CMP_FNS {
            if path_matches_global_names(
                func_path,
                &["std", "cmp", bit_result_trait, bit_result_fn],
            ) {
                return Some(boolean_type("concr"));
            }
        }

        // --- EXT ---

        if path_matches_global_names(func_path, &["machine_check", "Ext", "ext"]) {
            // infer from first argument and generic const
            let arg = &expr_call.args[0];
            // take the type from first typed argument we find
            let arg_ident = extract_expr_ident(arg).expect("Call argument should be ident");
            let arg_type = self
                .local_ident_types
                .get(arg_ident)
                .expect("Call argument should have local ident");

            let Some(Type::Path(ty_path)) = arg_type else {
                return None;
            };
            if !is_bitvector_related_path(&ty_path.path) {
                return None;
            }

            if !matches!(
                &func_path.segments[1].arguments,
                PathArguments::AngleBracketed(_)
            ) {
                return None;
            };
            // change generics
            let mut ty_path = ty_path.clone();
            ty_path.path.segments[1].arguments = func_path.segments[1].arguments.clone();

            return Some(Type::Path(ty_path));
        }

        /*
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
        let right_ident = extract_path_ident(&expr_path.path)
            .expect("Right side of assignment should be ident on path");
        let right_type = self
            .local_ident_types
            .get(right_ident)
            .expect("Right ident should be in ident types");
        right_type.clone()
    }

    fn infer_reference_result_type(&self, expr_reference: &ExprReference) -> Option<Type> {
        if let Expr::Field(expr_field) = expr_reference.expr.as_ref() {
            let Some(field_result_type) = self.infer_field_result_type(expr_field) else {
                return None;
            };

            return Some(Type::Reference(TypeReference {
                and_token: Default::default(),
                lifetime: None,
                mutability: None,
                elem: Box::new(field_result_type.clone()),
            }));
        }
        // infer type from the identifier first
        let expr_ident = extract_expr_ident(expr_reference.expr.as_ref())
            .expect("Right side of assignment should be ident in reference");
        let expr_type = self
            .local_ident_types
            .get(expr_ident)
            .expect("Right ident should be in ident types")
            .clone();

        let Some(expr_type) = expr_type else {
            return None;
        };

        // apply fields

        // resolve to reference
        Some(Type::Reference(TypeReference {
            and_token: Default::default(),
            lifetime: None,
            mutability: None,
            elem: Box::new(expr_type.clone()),
        }))
    }
}

fn is_bitvector_related_path(path: &Path) -> bool {
    path_matches_global_names(path, &["machine_check", "Bitvector"])
        || path_matches_global_names(path, &["machine_check", "Unsigned"])
        || path_matches_global_names(path, &["machine_check", "Signed"])
}
