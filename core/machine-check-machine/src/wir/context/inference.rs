use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

use indexmap::IndexMap;
use proc_macro2::Span;
use syn::Type;
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

use crate::{
    into_wir::{fold_type, Error, ErrorType},
    wir::{
        bitvector_type, bool_type,
        context::typedef::{WContextTypeDef, WTypeDefs},
        WIdent, WInferredContext, WItemImpl, WItemStruct, WPartialArgument, WPartialGenerics,
        WPartialPath, WPartialSegment, WPartialType, WSignatures, WSpan, WSubproperty, WTypeId,
        YTac,
    },
};

mod constraints;

#[derive(Debug)]
pub struct WInferenceContext {
    type_defs: WTypeDefs,
    types: Vec<WPartialType>,
    eq_constraints: QuickUnionUf<UnionBySize>,
}

impl WInferenceContext {
    pub fn new() -> Self {
        Self {
            type_defs: WTypeDefs::new(),
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
        if !ty.is_fully_inferred() {
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

    pub fn add_struct_def(&mut self, ty: Type, def: &WItemStruct) {
        let fields = def
            .fields
            .iter()
            .map(|field| (field.ident.clone(), field.ty.clone()))
            .collect();
        self.type_defs.add(ty, WContextTypeDef::Struct(fields));
    }

    fn add_eq_constraint(&mut self, a: WTypeId, b: WTypeId) {
        let max = a.0.max(b.0);
        while max >= self.eq_constraints.size() {
            self.eq_constraints.insert(UnionBySize::default());
        }

        self.eq_constraints.union(a.0, b.0);
    }

    pub fn infer_impls(
        mut self,
        signatures: WSignatures,
        impls: &[WItemImpl<YTac>],
    ) -> Result<WInferredContext, Error> {
        for item_impl in impls.iter() {
            for item_fn in &item_impl.impl_item_fns {
                let mut types = BTreeMap::new();

                for arg in &item_fn.signature.inputs {
                    types.insert(arg.ident.clone(), arg.ty.clone());
                }
                types.extend(
                    item_fn
                        .locals
                        .iter()
                        .map(|local| (local.ident.clone(), local.ty.clone())),
                );

                self.add_block_constraints(&signatures, &types, &item_fn.block)?;
            }
        }

        self.unify(signatures)
    }

    pub fn infer_subproperties(
        mut self,
        signatures: WSignatures,
        globals: &BTreeMap<WIdent, WTypeId>,
        subproperties: &[WSubproperty<YTac>],
    ) -> Result<WInferredContext, Error> {
        let mut globals_with_results = globals.clone();
        for (index, _) in subproperties.iter().enumerate() {
            globals_with_results.insert(
                WIdent::new(format!("__mck_subproperty_{}", index), Span::call_site()),
                self.partial_type_id(WPartialType::Path(WPartialPath {
                    leading_colon: None,
                    segments: vec![WPartialSegment {
                        ident: WIdent::new(String::from("bool"), Span::call_site()),
                        generics: None,
                    }],
                })),
            );
        }

        for subproperty in subproperties.iter() {
            match subproperty {
                WSubproperty::Func(subproperty_func) => {
                    let func = &subproperty_func.func;
                    let mut types = globals_with_results.clone();
                    types.extend(
                        func.locals
                            .iter()
                            .map(|local| (local.ident.clone(), local.ty.clone())),
                    );

                    self.add_block_constraints(&signatures, &types, &func.block)?;
                }
                WSubproperty::FixedPoint(_) => {}
                WSubproperty::Next(_) => {}
            }
        }

        self.unify(signatures)
    }

    fn unify(mut self, signatures: WSignatures) -> Result<WInferredContext, Error> {
        eprintln!("Unifying {:?}", self);
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

        self.into_total(signatures)
    }

    pub fn into_total(mut self, signatures: WSignatures) -> Result<WInferredContext, Error> {
        let boolean_type_id = self.type_id(&Type::from(bool_type()))?;
        let panic_type_id = self.type_id(&Type::from(bitvector_type(Some(32))))?;

        let mut types = Vec::new();
        for ty in self.types {
            let span = ty.wir_span();
            match ty.into_total() {
                Ok(ty) => types.push(ty),
                Err(()) => return Err(Error::new(ErrorType::InferenceFailure, span)),
            }
        }

        Ok(WInferredContext::new(
            signatures,
            self.type_defs,
            types,
            boolean_type_id,
            panic_type_id,
        ))
    }
}

fn join_types(previous: &WPartialType, current: WPartialType) -> Result<WPartialType, Error> {
    eprintln!("Joining types {:?} and {:?}", previous, current);
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
                        if lhs.turbofish.is_some() != rhs.turbofish.is_some() {
                            return Err(Error::new(ErrorType::InferenceFailure, span));
                        }

                        let mut arguments = Vec::new();
                        for (lhs, rhs) in lhs.arguments.iter().zip(rhs.arguments.into_iter()) {
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
                                (WPartialArgument::Type(lhs), WPartialArgument::Type(rhs)) => {
                                    WPartialArgument::Type(join_types(lhs, rhs)?)
                                }
                                _ => {
                                    return Err(Error::new(ErrorType::InferenceFailure, span));
                                }
                            };
                            arguments.push(arg);
                        }
                        Some(WPartialGenerics {
                            turbofish: rhs.turbofish,
                            arguments,
                        })
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
