use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

use indexmap::IndexMap;
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

use crate::{
    context::WInferredContext,
    wir::{
        WDefinitions, WPartialPath, WPartialPathArgument, WPartialPathGenerics,
        WPartialPathSegment, WPartialType, WTotalType, WTypeId, YTac,
    },
    Error, ErrorType,
};

mod constraints;

#[derive(Debug)]
pub struct WInferenceContext {
    definitions: WDefinitions<YTac>,
    types: Vec<WPartialType>,
    eq_constraints: QuickUnionUf<UnionBySize>,
}

impl WInferenceContext {
    pub fn new(definitions: WDefinitions<YTac>, types: Vec<WPartialType>) -> Self {
        Self {
            definitions,
            types,
            eq_constraints: QuickUnionUf::new(0),
        }
    }

    fn total_type_id(&mut self, ty: WTotalType) -> WTypeId {
        self.partial_type_id(ty.into_partial())
    }

    fn partial_type_id(&mut self, ty: WPartialType) -> WTypeId {
        let id = WTypeId::from_index(self.types.len());
        self.types.push(ty);
        id
    }

    fn add_eq_constraint(&mut self, a: WTypeId, b: WTypeId) {
        let max = a.index().max(b.index());
        while max >= self.eq_constraints.size() {
            self.eq_constraints.insert(UnionBySize::default());
        }

        self.eq_constraints.union(a.index(), b.index());
    }

    pub fn infer(mut self) -> Result<WInferredContext, Error> {
        for item_fn in self.definitions.functions().clone() {
            let mut types = BTreeMap::new();

            for arg in &item_fn.signature.inputs {
                types.insert(arg.ident.clone(), arg.ty.clone());
            }
            types.extend(
                item_fn
                    .body
                    .locals
                    .iter()
                    .map(|local| (local.ident.clone(), local.ty.clone())),
            );
            self.add_block_constraints(&types, &item_fn.body.block)?;
        }
        self.unify()
    }

    fn unify(mut self) -> Result<WInferredContext, Error> {
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

        self.into_total()
    }

    pub fn into_total(mut self) -> Result<WInferredContext, Error> {
        let boolean_type_id = self.total_type_id(WTotalType::new_bool());
        let panic_type_id = self.total_type_id(WTotalType::new_bitvector(Some(32)));

        let mut types = Vec::new();
        for ty in self.types {
            let span = ty.span();
            match ty.try_into_total() {
                Ok(ty) => types.push(ty),
                Err(()) => return Err(Error::new(ErrorType::InferenceFailure, span)),
            }
        }

        Ok(WInferredContext::new(
            self.definitions,
            types,
            boolean_type_id,
            panic_type_id,
        ))
    }
}

fn join_types(previous: &WPartialType, current: WPartialType) -> Result<WPartialType, Error> {
    eprintln!("Joining types {:?} and {:?}", previous, current);
    let span = current.span();
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
            for (lhs, rhs) in lhs.segments.iter().zip(rhs.segments) {
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
                        for (lhs, rhs) in lhs.arguments.iter().zip(rhs.arguments) {
                            let arg = match (lhs, rhs) {
                                (WPartialPathArgument::Infer(_), rhs) => rhs,
                                (lhs, WPartialPathArgument::Infer(_)) => lhs.clone(),
                                (
                                    WPartialPathArgument::Uint(lhs_num, _lhs_span),
                                    WPartialPathArgument::Uint(rhs_num, rhs_span),
                                ) => {
                                    if *lhs_num != rhs_num {
                                        return Err(Error::new(ErrorType::InferenceFailure, span));
                                    }
                                    WPartialPathArgument::Uint(rhs_num, rhs_span)
                                }
                                (
                                    WPartialPathArgument::Type(lhs),
                                    WPartialPathArgument::Type(rhs),
                                ) => WPartialPathArgument::Type(join_types(lhs, rhs)?),
                                _ => {
                                    return Err(Error::new(ErrorType::InferenceFailure, span));
                                }
                            };
                            arguments.push(arg);
                        }
                        Some(WPartialPathGenerics {
                            turbofish: rhs.turbofish,
                            arguments,
                        })
                    }
                };
                segments.push(WPartialPathSegment {
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
            return Err(Error::new(ErrorType::InferenceFailure, span));
        }
    })
}
