use std::fmt::Debug;

use indexmap::IndexMap;
use proc_macro2::Span;
use quote::quote;
use syn::{
    punctuated::Punctuated, spanned::Spanned, AngleBracketedGenericArguments, Expr, ExprInfer,
    GenericArgument, Ident, Path, PathArguments, PathSegment, Token, Type, TypeInfer, TypePath,
};

use crate::wir::{WPath, WTypeId};

#[derive(Clone, Debug)]
pub enum WContextTypeDef {
    Struct,
}

#[derive(Debug, Clone)]
pub enum WContextTypeResolved {
    Def(usize),
    Reference(Box<WContextTypeResolved>),
}

#[derive(Clone)]
pub enum WContextType {
    Unresolved(Box<Type>),
    Resolved(WContextTypeResolved),
}

impl Debug for WContextType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unresolved(ty) => {
                write!(f, "Unresolved({})", quote! (#ty))
            }
            Self::Resolved(resolved) => {
                write!(f, "Resolved({:?})", resolved)
            }
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WContextSynType(Type);

impl Debug for WContextSynType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ty = &self.0;
        write!(f, "{}", quote!(#ty))
    }
}

#[derive(Debug)]
pub struct WContext {
    type_defs: IndexMap<WContextSynType, WContextTypeDef>,
    types: Vec<WContextType>,
}

pub struct RequiresInferenceError;

impl WContext {
    pub fn new() -> Self {
        let mut type_defs = IndexMap::new();
        type_defs.insert(
            WContextSynType(Type::Path(TypePath {
                qself: None,
                path: Path {
                    leading_colon: Some(Token![::](Span::call_site())),
                    segments: Punctuated::from_iter([
                        PathSegment {
                            ident: Ident::new("machine_check", Span::call_site()),
                            arguments: PathArguments::None,
                        },
                        PathSegment {
                            ident: Ident::new("Bitvector", Span::call_site()),
                            arguments: PathArguments::AngleBracketed(
                                AngleBracketedGenericArguments {
                                    colon2_token: None,
                                    lt_token: Token![<](Span::call_site()),
                                    args: Punctuated::from_iter([GenericArgument::Const(
                                        Expr::Infer(ExprInfer {
                                            attrs: Vec::new(),
                                            underscore_token: Token![_](Span::call_site()),
                                        }),
                                    )]),
                                    gt_token: Token![>](Span::call_site()),
                                },
                            ),
                        },
                    ]),
                },
            })),
            WContextTypeDef::Struct,
        );
        Self {
            type_defs,
            types: Vec::new(),
        }
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

    pub fn infer_type(&mut self, span: Span) -> WTypeId {
        self.get_type(&Type::Infer(TypeInfer {
            underscore_token: Token![_](span),
        }))
    }

    pub fn add_struct_def(&mut self, ty: Type) {
        self.type_defs
            .insert(WContextSynType(ty), WContextTypeDef::Struct);
    }

    fn resolve_type(&self, mut ty: Type) -> Option<WContextTypeResolved> {
        if let Type::Reference(type_reference) = ty {
            return Some(WContextTypeResolved::Reference(Box::new(
                self.resolve_type(*type_reference.elem)?,
            )));
        }

        // strip out generics
        // TODO: preserve the generic data
        if let Type::Path(type_path) = &mut ty {
            for segment in &mut type_path.path.segments {
                match &mut segment.arguments {
                    PathArguments::AngleBracketed(bracketed) => {
                        for arg in &mut bracketed.args {
                            match arg {
                                GenericArgument::Type(_) => {
                                    *arg = GenericArgument::Type(Type::Infer(TypeInfer {
                                        underscore_token: Token![_](arg.span()),
                                    }));
                                }
                                GenericArgument::Const(_) => {
                                    *arg = GenericArgument::Const(Expr::Infer(ExprInfer {
                                        attrs: Vec::new(),
                                        underscore_token: Token![_](arg.span()),
                                    }));
                                }
                                _ => {}
                            }
                        }
                    }
                    PathArguments::None | PathArguments::Parenthesized(_) => {}
                }
            }
        }

        self.type_defs
            .get_index_of(&WContextSynType(ty))
            .map(WContextTypeResolved::Def)
            .map(|index| index)
    }

    pub fn resolve_types(&mut self) {
        for i in 0..self.types.len() {
            let context_type = &self.types[i];
            if let WContextType::Unresolved(ty) = context_type {
                if let Some(resolved) = self.resolve_type(*ty.clone()) {
                    self.types[i] = WContextType::Resolved(resolved);
                }
            }
        }
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
