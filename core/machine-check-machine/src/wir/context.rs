use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

use indexmap::IndexMap;
use proc_macro2::Span;
use quote::quote;
use syn::Type;
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

use crate::{
    into_wir::{fold_type, Error, ErrorType},
    wir::{
        WIdent, WItemImpl, WPartialArgument, WPartialPath, WPartialSegment, WPartialType, WSpan,
        WTypeId, YTac,
    },
};

mod constraints;

#[derive(Clone, Debug)]
pub enum WContextTypeDef {
    Struct,
}
/*
#[derive(Debug, Clone)]
pub enum WContextTypeResolved {
    Bool,
    Bitvector(Option<u32>),
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
}*/

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
    types: Vec<WPartialType>,
    eq_constraints: QuickUnionUf<UnionBySize>,
}

impl WContext {
    pub fn new() -> Self {
        Self {
            type_defs: IndexMap::new(),
            types: Vec::new(),
            eq_constraints: QuickUnionUf::new(0),
        }
    }

    fn partial_type_id(&mut self, ty: WPartialType) -> WTypeId {
        let id = WTypeId(self.types.len());
        self.types.push(ty);
        id
    }

    pub fn type_id(&mut self, ty: &Type) -> Result<WTypeId, Error> {
        let ty = fold_type(ty.clone())?;
        Ok(self.partial_type_id(ty))
    }

    pub fn noninferred_id(&mut self, ty: &Type) -> Result<WTypeId, Error> {
        let span = WSpan::from_syn(&ty);
        let ty = fold_type(ty.clone())?;
        if needs_inference(&ty) {
            return Err(Error::new(
                crate::into_wir::ErrorType::IllegalConstruct(String::from(
                    "Interference not allowed here",
                )),
                span,
            ));
        }
        Ok(self.partial_type_id(ty))
    }

    pub fn wildcard_id(&mut self, span: WSpan) -> WTypeId {
        self.partial_type_id(WPartialType::Infer(span))
    }

    pub fn add_struct_def(&mut self, ty: Type) {
        self.type_defs
            .insert(WContextSynType(ty), WContextTypeDef::Struct);
    }

    fn add_eq_constraint(&mut self, a: WTypeId, b: WTypeId) {
        let max = a.0.max(b.0);
        while max >= self.eq_constraints.size() {
            self.eq_constraints.insert(UnionBySize::default());
        }

        self.eq_constraints.union(a.0, b.0);
    }
    pub fn resolve_types(&mut self, impls: &[WItemImpl<YTac>]) -> Result<(), Error> {
        for item_impl in impls.iter() {
            for item_fn in &item_impl.impl_item_fns {
                self.add_block_constraints(&item_fn.locals, &item_fn.block)?;
            }
        }

        let mut united = IndexMap::new();

        for i in 0..self.types.len() {
            let current = self.types[i].clone();
            let root = self.eq_constraints.find(i);

            let next = if let Some(previous) = united.get(&root) {
                join_types(previous, current)?
            } else {
                current
            };

            united.insert(root, next);
        }

        let mut eq_classes = BTreeMap::<usize, BTreeSet<usize>>::new();

        for i in 0..self.types.len() {
            let root = self.eq_constraints.find(i);
            let resolved = united
                .get(&root)
                .expect("Equality class root should have type");
            self.types[i] = resolved.clone();
            eq_classes.entry(root).or_default().insert(i);
        }

        for i in 0..self.types.len() {
            let root = self.eq_constraints.find(i);

            eprintln!(
                "Type @{}: {:?} (root {}, equality class {:?})",
                i,
                &self.types[i],
                root,
                eq_classes.entry(root).or_default()
            );
        }

        Ok(())
    }
}

fn needs_inference(ty: &WPartialType) -> bool {
    eprintln!("Deciding if needs inference: {:?}", ty);
    match ty {
        WPartialType::Path(path) => {
            for segment in &path.segments {
                if let Some(arguments) = &segment.generics {
                    for argument in arguments {
                        match argument {
                            super::WPartialArgument::Uint(_, _) => {}
                            super::WPartialArgument::Infer(_) => return true,
                        }
                    }
                }
            }
            false
        }
        WPartialType::Infer(_) => true,
        WPartialType::Reference(inner) => needs_inference(inner),
    }
}

fn bitvector_type(width: Option<u32>) -> WPartialType {
    let generics = if let Some(width) = width {
        Some(vec![WPartialArgument::Uint(width, WSpan::call_site())])
    } else {
        None
    };
    WPartialType::Path(WPartialPath {
        leading_colon: Some(WSpan::call_site()),
        segments: vec![
            WPartialSegment {
                ident: WIdent::new(String::from("machine_check"), Span::call_site()),
                generics: None,
            },
            WPartialSegment {
                ident: WIdent::new(String::from("Bitvector"), Span::call_site()),
                generics,
            },
        ],
    })
}

fn bool_type() -> WPartialType {
    WPartialType::Path(WPartialPath {
        leading_colon: None,
        segments: vec![WPartialSegment {
            ident: WIdent::new(String::from("bool"), Span::call_site()),
            generics: None,
        }],
    })
}

fn join_types(previous: &WPartialType, current: WPartialType) -> Result<WPartialType, Error> {
    let span = current.wir_span();
    Ok(match (previous, current) {
        (WPartialType::Infer(_), current) => current,
        (previous, WPartialType::Infer(_)) => previous.clone(),
        (WPartialType::Path(lhs), WPartialType::Path(rhs)) => {
            if lhs.leading_colon.is_some() != rhs.leading_colon.is_some()
                || lhs.segments.len() != rhs.segments.len()
            {
                return Err(Error::new(ErrorType::InferenceFailure, span));
            }
            let mut segments = Vec::new();
            for (lhs, rhs) in lhs.segments.iter().zip(rhs.segments.into_iter()) {
                if lhs.ident != rhs.ident {
                    return Err(Error::new(ErrorType::InferenceFailure, span));
                }
                let generics = match (&lhs.generics, rhs.generics) {
                    (None, None) => None,
                    (None, Some(rhs)) => Some(rhs),
                    (Some(lhs), None) => Some(lhs.clone()),
                    (Some(lhs), Some(rhs)) => {
                        let mut arguments = Vec::new();
                        for (lhs, rhs) in lhs.iter().zip(rhs.into_iter()) {
                            let arg = match (lhs, rhs) {
                                (WPartialArgument::Infer(_), rhs) => rhs,
                                (lhs, WPartialArgument::Infer(_)) => lhs.clone(),
                                (
                                    WPartialArgument::Uint(lhs_num, _lhs_span),
                                    WPartialArgument::Uint(rhs_num, rhs_span),
                                ) => {
                                    if *lhs_num != rhs_num {
                                        return Err(Error::new(ErrorType::InferenceFailure, span));
                                    }
                                    WPartialArgument::Uint(rhs_num, rhs_span)
                                }
                            };
                            arguments.push(arg);
                        }
                        Some(arguments)
                    }
                };
                segments.push(WPartialSegment {
                    ident: rhs.ident,
                    generics,
                });
            }
            WPartialType::Path(WPartialPath {
                leading_colon: rhs.leading_colon,
                segments,
            })
        }
        (WPartialType::Reference(lhs), WPartialType::Reference(rhs)) => {
            let joined_inner = join_types(lhs, *rhs)?;
            WPartialType::Reference(Box::new(joined_inner))
        }
        (_previous, _current) => {
            return Err(Error::new(
                crate::into_wir::ErrorType::InferenceFailure,
                span,
            ));
        }
    })
}
