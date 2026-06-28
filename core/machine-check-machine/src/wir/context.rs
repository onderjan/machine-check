use std::fmt::Debug;

use indexmap::IndexMap;
use machine_check_common::ir_common::IrStdBinaryOp;
use proc_macro2::Span;
use quote::quote;
use syn::{
    punctuated::Punctuated, spanned::Spanned, AngleBracketedGenericArguments, Expr, ExprInfer,
    GenericArgument, Ident, Path, PathArguments, PathSegment, Token, Type, TypeInfer, TypePath,
};
use syn_path::path;
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

use crate::wir::{
    WBlock, WExpr, WExprHighCall, WIndexedExpr, WIndexedIdent, WItemImpl, WItemStruct,
    WMacroableStmt, WPath, WTacLocal, WTypeId, YTac, ZTac,
};

#[derive(Clone, Debug)]
pub enum WContextTypeDef {
    Bool,
    Bitvector,
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
    eq_constraints: QuickUnionUf<UnionBySize>,
}

pub struct RequiresInferenceError;

impl WContext {
    pub fn new() -> Self {
        let mut type_defs = IndexMap::new();
        type_defs.insert(WContextSynType(Self::bool_type()), WContextTypeDef::Bool);
        type_defs.insert(
            WContextSynType(Self::bitvector_type()),
            WContextTypeDef::Bitvector,
        );
        Self {
            type_defs,
            types: Vec::new(),
            eq_constraints: QuickUnionUf::new(0),
        }
    }

    fn bitvector_type() -> Type {
        Type::Path(TypePath {
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
                        arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: Token![<](Span::call_site()),
                            args: Punctuated::from_iter([GenericArgument::Const(Expr::Infer(
                                ExprInfer {
                                    attrs: Vec::new(),
                                    underscore_token: Token![_](Span::call_site()),
                                },
                            ))]),
                            gt_token: Token![>](Span::call_site()),
                        }),
                    },
                ]),
            },
        })
    }

    fn bool_type() -> Type {
        Type::Path(TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: Punctuated::from_iter([PathSegment {
                    ident: Ident::new("bool", Span::call_site()),
                    arguments: PathArguments::None,
                }]),
            },
        })
    }

    pub fn type_def_index(&mut self, ty: &Type) -> WTypeId {
        WTypeId(
            self.type_defs
                .get_index_of(&WContextSynType(ty.clone()))
                .expect("Type should be in type defs"),
        )
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
    }

    fn add_eq_constraint(&mut self, a: WTypeId, b: WTypeId) {
        let max = a.0.max(b.0);
        while max >= self.eq_constraints.size() {
            self.eq_constraints.insert(UnionBySize::default());
        }

        self.eq_constraints.union(a.0, b.0);
    }
    fn add_block_constraints(&mut self, locals: &Vec<WTacLocal<WTypeId>>, block: &WBlock<ZTac>) {
        for stmt in &block.stmts {
            eprintln!("Should add constraints for statement {:#?}", stmt);
            match stmt {
                WMacroableStmt::Assign(assign) => {
                    let left = match &assign.left {
                        WIndexedIdent::Indexed(wident, wident1) => {
                            todo!("Constraints for indexed left")
                        }
                        WIndexedIdent::NonIndexed(ident) => ident,
                    };
                    let right = match &assign.right {
                        WIndexedExpr::Indexed(base_expr, ident) => {
                            todo!("Constraints for indexed right")
                        }
                        WIndexedExpr::NonIndexed(expr) => expr,
                    };
                    eprintln!(
                        "Should add constraints for left {:?}, right {:?}",
                        left, right
                    );

                    let left_ty = locals
                        .iter()
                        .find(|e| &e.ident == left)
                        .expect("Local should be found")
                        .ty
                        .clone();

                    match right {
                        WExpr::Move(wident) => todo!("Move"),
                        WExpr::Call(call) => {
                            eprintln!("Call");
                            match call {
                                WExprHighCall::Call(call) => {
                                    let bitvector_new_path = path!(::machine_check::Bitvector::new);
                                    let fn_path = Path::from(call.fn_path.clone());
                                    if fn_path == bitvector_new_path {
                                        // constrain the output to be a bitvector
                                        let bitvector_ty = self.get_type(&Self::bitvector_type());
                                        self.add_eq_constraint(left_ty, bitvector_ty);

                                        eprintln!("Bitvector new");
                                    } else {
                                        todo!("Call")
                                    }
                                }
                                WExprHighCall::StdUnary(unary) => todo!("Std unary"),
                                WExprHighCall::StdBinary(binary) => {
                                    let a_ty = locals
                                        .iter()
                                        .find(|e| e.ident == binary.a)
                                        .expect("Local should be found")
                                        .ty
                                        .clone();

                                    let b_ty = locals
                                        .iter()
                                        .find(|e| e.ident == binary.b)
                                        .expect("Local should be found")
                                        .ty
                                        .clone();
                                    match binary.op {
                                        IrStdBinaryOp::Eq => {
                                            // constrain the inputs to be of the same type
                                            self.add_eq_constraint(a_ty, b_ty);
                                            // constrain the output to be a Boolean
                                            let bool_ty = self.get_type(&Self::bool_type());
                                            self.add_eq_constraint(left_ty, bool_ty);
                                        }
                                        IrStdBinaryOp::Add => {
                                            // constrain both inputs to output
                                            self.add_eq_constraint(left_ty.clone(), a_ty);
                                            self.add_eq_constraint(left_ty, b_ty);
                                        }
                                        _ => todo!("Std binary"),
                                    }
                                }
                            }
                        }
                        WExpr::Field(wexpr_field) => {
                            eprintln!("Field");
                        }
                        WExpr::Struct(expr_struct) => {
                            let struct_ty = self.get_type(&Type::Path(TypePath {
                                qself: None,
                                path: Path::from(expr_struct.type_path.clone()),
                            }));
                            self.add_eq_constraint(left_ty, struct_ty);
                        }
                        WExpr::Reference(wexpr_reference) => todo!("Reference"),
                        WExpr::Lit(lit, _) => todo!("Literal"),
                    }
                }
                WMacroableStmt::If(stmt_if) => {
                    eprintln!("Should add constraints for if {:#?}", stmt_if);
                    self.add_block_constraints(locals, &stmt_if.then_block);
                    self.add_block_constraints(locals, &stmt_if.else_block);
                }
                WMacroableStmt::PanicMacro(wstmt_panic_macro) => {
                    todo!("Constraints for panic macro")
                }
            }
        }
    }

    pub fn resolve_types(&mut self, structs: &[WItemStruct<WTypeId>], impls: &[WItemImpl<YTac>]) {
        for item_impl in impls.iter() {
            for item_fn in &item_impl.impl_item_fns {
                self.add_block_constraints(&item_fn.locals, &item_fn.block);
            }
        }

        let mut united = IndexMap::new();

        for i in 0..self.types.len() {
            let context_type = &self.types[i];
            let root = self.eq_constraints.find(i);
            match context_type {
                WContextType::Unresolved(unresolved) => {
                    if let Some(resolved) = self.resolve_type(*unresolved.clone()) {
                        united.insert(root, resolved);
                    }
                }
                WContextType::Resolved(resolved) => {
                    united.insert(root, resolved.clone());
                }
            }
            if let WContextType::Unresolved(ty) = context_type {}
        }

        eprintln!("United resolved: {:?}", united);

        for i in 0..self.types.len() {
            let root = self.eq_constraints.find(i);
            if let Some(resolved) = united.get(&root) {
                self.types[i] = WContextType::Resolved(resolved.clone());
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
