use core::panic;
use std::collections::HashMap;

use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    AngleBracketedGenericArguments, Expr, ExprCall, ExprField, ExprPath, ExprReference,
    GenericArgument, Ident, ItemStruct, Member, Path, PathArguments, Token, Type, TypeReference,
};

use crate::{
    util::{
        create_type_path, extract_expr_ident, extract_path_ident, extract_type_path,
        path_matches_global_names,
    },
    ErrorType, MachineError,
};

use super::type_properties::is_type_fully_specified;

mod infer_call;

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

        let Some(ty) = self.local_ident_types.get_mut(left_ident) else {
            // not a local ident, skip
            return;
        };

        // check whether the left type has already a determined left type
        if let Some(ty) = ty {
            if is_type_fully_specified(ty) {
                // we already have determined left type
                // try to infer PanicResult type if it is field base

                if let Expr::Field(right_field) = expr_assign.right.as_ref() {
                    let left_type = ty.clone();
                    let base_ident = extract_expr_ident(right_field.base.as_ref())
                        .expect("Right field base should be an ident");
                    if let Some(Some(Type::Path(base_type_path))) =
                        self.local_ident_types.get_mut(base_ident)
                    {
                        if path_matches_global_names(
                            &base_type_path.path,
                            &["machine_check", "internal", "PanicResult"],
                        ) {
                            let generics_span = base_type_path.path.segments[2].span();
                            // infer the generics
                            base_type_path.path.segments[2].arguments =
                                PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                                    colon2_token: Some(Default::default()),
                                    lt_token: Token![<](generics_span),
                                    args: Punctuated::from_iter([GenericArgument::Type(left_type)]),
                                    gt_token: Token![>](generics_span),
                                });
                        }
                    };
                }
                return;
            }
        }

        let inferred_type = match expr_assign.right.as_ref() {
            syn::Expr::Path(ref right_path) => self.infer_path_result_type(right_path),
            syn::Expr::Call(ref right_call) => self.infer_call_result_type(right_call),
            syn::Expr::Field(ref right_field) => self.infer_field_result_type(right_field),
            syn::Expr::Reference(ref right_reference) => {
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
                *self.local_ident_types.get_mut(left_ident).unwrap() = Some(inferred_type);
                self.inferred_something = true;
            }
        }

        // delegate visit
        visit_mut::visit_expr_assign_mut(self, expr_assign);
    }
}

impl LocalVisitor<'_> {
    fn push_error(&mut self, error: MachineError) {
        if self.result.is_ok() {
            self.result = Err(error);
        }
    }

    fn infer_field_result_type(&self, expr_field: &ExprField) -> Option<Type> {
        // get type of member from structs
        let base_ident =
            extract_expr_ident(expr_field.base.as_ref()).expect("Field base should be an ident");

        let Some(base_type) = self.local_ident_types.get(base_ident) else {
            // not a local ident, skip
            return None;
        };
        let mut base_type: &Type = base_type.as_ref()?;
        // dereference first
        while let Type::Reference(ref_type) = base_type {
            base_type = ref_type.elem.as_ref();
        }

        let Some(base_type_path) = extract_type_path(base_type) else {
            panic!("Unexpected non-path base type");
        };

        let base_struct = self.structs.get(&base_type_path);
        let base_struct = base_struct?;
        match &base_struct.fields {
            syn::Fields::Named(fields) => {
                // match ident
                let Member::Named(member_ident) = &expr_field.member else {
                    return None;
                };
                let field = fields.named.iter().find(|field| {
                    let field_ident = field.ident.as_ref().unwrap();
                    field_ident == member_ident
                })?;
                Some(field.ty.clone())
            }
            syn::Fields::Unnamed(fields) => {
                let Member::Unnamed(member_index) = &expr_field.member else {
                    return None;
                };
                let field = fields.unnamed.iter().nth(member_index.index as usize)?;
                Some(field.ty.clone())
            }
            syn::Fields::Unit => None,
        }
    }

    fn infer_path_result_type(&self, expr_path: &ExprPath) -> Option<Type> {
        // infer from the identifier
        let right_ident = extract_path_ident(&expr_path.path)
            .expect("Right side of assignment should be ident on path");
        let right_type = self.local_ident_types.get(right_ident);
        if let Some(right_type) = right_type {
            right_type.clone()
        } else {
            None
        }
    }

    fn infer_reference_result_type(&self, expr_reference: &ExprReference) -> Option<Type> {
        if let Expr::Field(ref expr_field) = expr_reference.expr.as_ref() {
            let field_result_type = self.infer_field_result_type(expr_field)?;

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
        let expr_type = self.local_ident_types.get(expr_ident);

        let Some(Some(expr_type)) = expr_type else {
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

    fn get_arg_type<'a>(
        &'a mut self,
        expr_call: &ExprCall,
        arg_index: usize,
        num_args: usize,
    ) -> Result<Option<&'a Type>, ()> {
        assert!(arg_index < num_args);
        if num_args != expr_call.args.len() {
            self.push_error(MachineError::new(
                ErrorType::UnsupportedConstruct(format!(
                    "Expected {} parameters, but {} supplied",
                    num_args,
                    expr_call.args.len()
                )),
                expr_call.span(),
            ));
            return Err(());
        }
        let arg = &expr_call.args[arg_index];
        let arg_ident = extract_expr_ident(arg).expect("Call argument should be ident");
        let arg_type = self.local_ident_types.get(arg_ident);
        if let Some(arg_type) = arg_type {
            Ok(arg_type.as_ref())
        } else {
            // not a local ident, do not produce an error
            Ok(None)
        }
    }
}
