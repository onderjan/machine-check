use proc_macro2::Span;
use syn::{GenericArgument, Token, Type, TypeInfer};

use crate::wir::{WPath, WTypeId};

#[derive(Clone, Debug)]
pub enum WContextType {
    Unresolved(Box<Type>),
    Resolved,
}

#[derive(Debug)]
pub struct WContext {
    types: Vec<WContextType>,
}

pub struct RequiresInferenceError;

impl WContext {
    pub fn new() -> Self {
        Self { types: Vec::new() }
    }

    pub fn get_type(&mut self, ty: &Type) -> WTypeId {
        let id = WTypeId(self.types.len());
        self.types
            .push(WContextType::Unresolved(Box::new(ty.clone())));
        id
    }

    pub fn get_noninferred_type(&mut self, ty: &Type) -> Result<WTypeId, RequiresInferenceError> {
        if let Some(false) = needs_inference(ty) {
            Ok(self.get_type(ty))
        } else {
            Err(RequiresInferenceError)
        }
    }

    pub fn get_noninferred_type_path(
        &mut self,
        path: &WPath,
    ) -> Result<WTypeId, RequiresInferenceError> {
        let id = WTypeId(self.types.len());
        // TODO: check that it is noninferred
        Ok(id)
    }

    pub fn infer_type(&mut self, span: Span) -> WTypeId {
        self.get_type(&Type::Infer(TypeInfer {
            underscore_token: Token![_](span),
        }))
    }
}

fn needs_inference(ty: &Type) -> Option<bool> {
    match ty {
        Type::Group(type_group) => needs_inference(&type_group.elem),
        Type::Infer(_) => Some(true),
        Type::Paren(type_paren) => needs_inference(&type_paren.elem),
        Type::Path(type_path) => {
            for segment in &type_path.path.segments {
                match &segment.arguments {
                    syn::PathArguments::None => {}
                    syn::PathArguments::AngleBracketed(bracketed) => {
                        for arg in &bracketed.args {
                            match arg {
                                GenericArgument::Type(ty) => match needs_inference(ty) {
                                    Some(false) => {}
                                    x => return x,
                                },
                                GenericArgument::Const(arg_const) => match arg_const {
                                    syn::Expr::Lit(_) => {}
                                    _ => {
                                        // unsupported
                                        return None;
                                    }
                                },
                                GenericArgument::Lifetime(_)
                                | GenericArgument::AssocType(_)
                                | GenericArgument::AssocConst(_)
                                | GenericArgument::Constraint(_)
                                | _ => {
                                    // unsupported
                                    return None;
                                }
                            }
                        }
                    }
                    syn::PathArguments::Parenthesized(_) => {
                        // unsupported
                        return None;
                    }
                }
            }
            Some(false)
        }
        Type::Reference(type_reference) => needs_inference(&type_reference.elem),
        Type::Tuple(type_tuple) => {
            for elem_type in &type_tuple.elems {
                match needs_inference(elem_type) {
                    Some(false) => {}
                    x => return x,
                }
            }
            Some(false)
        }
        Type::Array(_)
        | Type::BareFn(_)
        | Type::ImplTrait(_)
        | Type::Macro(_)
        | Type::Never(_)
        | Type::Ptr(_)
        | Type::Slice(_)
        | Type::TraitObject(_)
        | Type::Verbatim(_)
        | _ => {
            // unsupported
            None
        }
    }
}
