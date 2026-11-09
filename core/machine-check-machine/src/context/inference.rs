use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

use indexmap::IndexMap;
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

use crate::{
    context::WTypedContext,
    wir::{
        WDefinitions, WIdent, WInferenceType, WSpan, WStmtAssign, WType, WTypeId, WTypePath,
        WTypePathSegment, YTac,
    },
    Error, ErrorType,
};

mod constraints;

#[derive(Debug, PartialEq, Eq, Clone)]
enum WConstraint {
    Assign(WStmtAssign<YTac>),
}

#[derive(Debug, Clone)]
pub struct WFunctionConstraints {
    eq: QuickUnionUf<UnionBySize>,
    noneq: Vec<WConstraint>,
}

#[derive(Debug)]
pub struct WInferenceContext {
    definitions: WDefinitions<YTac>,
    types: Vec<WInferenceType>,
    fn_constraints: Vec<WFunctionConstraints>,

    boolean_type_id: WTypeId,
    panic_type_id: WTypeId,
}

impl WInferenceContext {
    pub fn new(
        definitions: WDefinitions<YTac>,
        types: Vec<WInferenceType>,
        boolean_type_id: WTypeId,
        panic_type_id: WTypeId,
    ) -> Self {
        let fn_constraints = vec![
            WFunctionConstraints {
                eq: QuickUnionUf::new(0),
                noneq: Vec::new()
            };
            definitions.functions().len()
        ];
        Self {
            definitions,
            types,
            fn_constraints,
            boolean_type_id,
            panic_type_id,
        }
    }

    fn known_type_id(&mut self, ty: WType) -> WTypeId {
        self.partial_type_id(WInferenceType::Inferred(ty))
    }

    fn partial_type_id(&mut self, ty: WInferenceType) -> WTypeId {
        let id = WTypeId::from_index(self.types.len());
        self.types.push(ty);
        id
    }

    fn add_eq_constraint(&mut self, fn_index: usize, a: WTypeId, b: WTypeId) {
        let constraints = &mut self.fn_constraints[fn_index];
        let max = a.index().max(b.index());
        while max >= constraints.eq.size() {
            constraints.eq.insert(UnionBySize::default());
        }

        constraints.eq.union(a.index(), b.index());
    }

    fn add_assignment_constraint(&mut self, fn_index: usize, stmt: &WStmtAssign<YTac>) {
        let constraints = &mut self.fn_constraints[fn_index];
        constraints.noneq.push(WConstraint::Assign(stmt.clone()));
    }

    pub fn infer(mut self) -> Result<WTypedContext, Error> {
        for fn_index in 0..self.definitions.functions().len() {
            self.infer_fn(fn_index)?;
        }

        self.into_total()
    }

    pub fn infer_fn(&mut self, fn_index: usize) -> Result<(), Error> {
        let item_fn = &self.definitions.functions()[fn_index];

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
        self.resolve_block_constraints(fn_index, &types, &item_fn.body.block.clone())?;

        const MAX_ITERATIONS: u32 = 256;

        let mut i = 0;
        loop {
            i += 1;

            if i >= MAX_ITERATIONS {
                panic!("Too many inference iterations");
            }

            eprintln!("Inference iteration #{i}");

            self.unify_fn(fn_index)?;

            let noneq_constraints = std::mem::take(&mut self.fn_constraints[fn_index].noneq);

            eprintln!(
                "Unified, resolving non-equality constraints: {:?}",
                noneq_constraints
            );

            for constraint in &noneq_constraints {
                match constraint {
                    WConstraint::Assign(assign) => {
                        self.resolve_assignment_constraint(fn_index, &types, assign)?;
                    }
                }
            }

            let new_noneq_constraints = &self.fn_constraints[fn_index].noneq;

            eprintln!(
                "Resolved non-equality constraints: {:?}",
                new_noneq_constraints
            );

            if new_noneq_constraints == &noneq_constraints {
                eprintln!("Inference fixpoint reached");
                return Ok(());
            }
        }
    }

    fn unify_fn(&mut self, fn_index: usize) -> Result<(), Error> {
        let constraints = &mut self.fn_constraints[fn_index].eq;
        eprintln!("Unifying {:?}", constraints);

        while self.types.len() >= constraints.size() {
            constraints.insert(UnionBySize::default());
        }

        let mut united = IndexMap::new();

        for i in 0..self.types.len() {
            let current = self.types[i].clone();
            let root = constraints.find(i);

            let next = if let Some(previous) = united.get(&root) {
                join_type(&current, previous)?
            } else {
                current
            };

            united.insert(root, next);
        }

        let mut eq_classes = BTreeMap::<usize, BTreeSet<usize>>::new();

        for i in 0..self.types.len() {
            let root = constraints.find(i);
            let resolved = united
                .get(&root)
                .expect("Equality class root should have type");
            self.types[i] = resolved.clone();
            eq_classes.entry(root).or_default().insert(i);
        }

        for i in 0..self.types.len() {
            let root = constraints.find(i);

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

    pub fn into_total(self) -> Result<WTypedContext, Error> {
        let mut types = Vec::new();
        for ty in self.types {
            let span = ty.span();
            match ty.try_into_total() {
                Ok(ty) => types.push(ty),
                Err(()) => return Err(Error::new(ErrorType::InferenceFailure, span)),
            }
        }

        Ok(WTypedContext::new(
            self.definitions,
            types,
            self.boolean_type_id,
            self.panic_type_id,
        ))
    }

    pub fn new_bitvector(&mut self, width: Option<u32>) -> WTypeId {
        self.new_bitvector_like("Bitvector", width)
    }

    pub fn new_unsigned(&mut self, width: Option<u32>) -> WTypeId {
        self.new_bitvector_like("Unsigned", width)
    }

    pub fn new_signed(&mut self, width: Option<u32>) -> WTypeId {
        self.new_bitvector_like("Signed", width)
    }

    fn new_bitvector_like(&mut self, name: &str, width: Option<u32>) -> WTypeId {
        //let arg = WPartialPathArgument::Uint(width, WSpan::call_site());
        let generics = if let Some(width) = width {
            vec![self.known_type_id(WType::Number(width, WSpan::call_site()))]
        } else {
            vec![]
        };
        let ty = WType::Path(WTypePath {
            leading_colon: Some(WSpan::call_site()),
            segments: vec![
                WTypePathSegment {
                    ident: WIdent::new(String::from("machine_check"), WSpan::call_site()),
                    generics: None,
                },
                WTypePathSegment {
                    ident: WIdent::new(String::from(name), WSpan::call_site()),
                    generics: Some(generics),
                },
            ],
        });

        self.known_type_id(ty)
    }

    pub fn new_bool(&mut self) -> WTypeId {
        let ty = WType::Path(WTypePath {
            leading_colon: None,
            segments: vec![WTypePathSegment {
                ident: WIdent::new(String::from("bool"), WSpan::call_site()),
                generics: None,
            }],
        });
        self.known_type_id(ty)
    }
}

fn join_type(lhs: &WInferenceType, rhs: &WInferenceType) -> Result<WInferenceType, Error> {
    let span = lhs.span();
    match (lhs, rhs) {
        (WInferenceType::Infer(_), ty) | (ty, WInferenceType::Infer(_)) => Ok(ty.clone()),
        (WInferenceType::Inferred(lhs), WInferenceType::Inferred(rhs)) => {
            let result = match (lhs, rhs) {
                (WType::Path(lhs), WType::Path(rhs)) => {
                    let span = lhs.clone().without_generics().span();
                    if lhs.leading_colon.is_some() != rhs.leading_colon.is_some()
                        || lhs.segments.len() != rhs.segments.len()
                    {
                        return Err(Error::new(ErrorType::InferenceFailure, span));
                    }

                    let mut segments = Vec::new();

                    for (lhs, rhs) in lhs.segments.iter().zip(rhs.segments.iter()) {
                        if lhs.ident != rhs.ident {
                            return Err(Error::new(ErrorType::InferenceFailure, span));
                        }

                        let generics = match (&lhs.generics, &rhs.generics) {
                            (None, None) => None,
                            (Some(generics), None) | (None, Some(generics)) => {
                                Some(generics.clone())
                            }
                            (Some(lhs), Some(rhs)) => {
                                if lhs.len() != rhs.len() {
                                    return Err(Error::new(ErrorType::InferenceFailure, span));
                                }
                                // do not go into the types
                                Some(lhs.clone())
                            }
                        };

                        segments.push(WTypePathSegment {
                            ident: lhs.ident.clone(),
                            generics,
                        });
                    }

                    WType::Path(WTypePath {
                        leading_colon: lhs.leading_colon,
                        segments,
                    })
                }
                (WType::Reference(lhs, _), WType::Reference(_rhs, _)) => {
                    /*let lhs = self.types[lhs.index()].clone();
                    let rhs = self.types[rhs.index()].clone();
                    let joined = self.join_type(&lhs, &rhs);*/
                    WType::Reference(lhs.clone(), span)
                }
                (WType::Number(lhs, span), WType::Number(rhs, _)) => {
                    if lhs != rhs {
                        return Err(Error::new(ErrorType::InferenceFailure, *span));
                    }
                    WType::Number(*lhs, *span)
                }
                _ => return Err(Error::new(ErrorType::InferenceFailure, span)),
            };
            Ok(WInferenceType::Inferred(result))
        }
    }
}
