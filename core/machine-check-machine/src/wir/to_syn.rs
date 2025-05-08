use quote::ToTokens;
use syn::{
    punctuated::Punctuated,
    token::{Brace, Bracket, Comma, Paren},
    AngleBracketedGenericArguments, Attribute, Block, Expr, ExprAssign, ExprBlock, ExprCall,
    ExprField, ExprIf, ExprLit, ExprReference, ExprStruct, Field, FieldValue, FieldsNamed, FnArg,
    GenericArgument, Generics, Ident, ImplItem, ImplItemFn, ImplItemType, Item, ItemImpl,
    ItemStruct, Lit, LitInt, Local, MetaList, MetaNameValue, Pat, PatIdent, PatType, Path,
    PathArguments, PathSegment, Receiver, Signature, Stmt, Token, Type, TypePath, TypeReference,
    Visibility,
};
use syn_path::path;

use crate::util::{create_expr_ident, create_expr_path, create_path_from_ident};

use super::*;

impl WDescription {
    pub fn into_syn(self) -> impl Iterator<Item = Item> {
        self.items.into_iter().map(fold_item)
    }
}

fn fold_item(item: WItem) -> Item {
    match item {
        WItem::Struct(item) => Item::Struct(fold_item_struct(item)),
        WItem::Impl(item) => Item::Impl(fold_item_impl(item)),
    }
}

fn fold_item_struct(item: WItemStruct) -> ItemStruct {
    let span = Span::call_site();

    let named = Punctuated::from_iter(item.fields.into_iter().map(|field| Field {
        attrs: Vec::new(),
        // TODO visibility
        vis: syn::Visibility::Inherited,
        mutability: syn::FieldMutability::None,
        ident: Some(field.ident.into()),
        colon_token: Some(Token![:](span)),
        ty: fold_simple_type(field.ty),
    }));

    let fields = FieldsNamed {
        brace_token: Brace::default(),
        named,
    };

    let mut attrs = Vec::new();

    if !item.derives.is_empty() {
        let derive_tokens =
            Punctuated::<Path, Comma>::from_iter(item.derives.into_iter().map(Path::from))
                .into_token_stream();

        let derive_attribute = Attribute {
            pound_token: Token![#](span),
            style: syn::AttrStyle::Outer,
            bracket_token: Bracket::default(),
            meta: syn::Meta::List(MetaList {
                path: Path {
                    leading_colon: None,
                    segments: Punctuated::from_iter([PathSegment {
                        ident: Ident::new("derive", span),
                        arguments: syn::PathArguments::None,
                    }]),
                },
                delimiter: syn::MacroDelimiter::Paren(Paren::default()),
                tokens: derive_tokens,
            }),
        };

        attrs.push(derive_attribute);
    }

    ItemStruct {
        attrs,
        // TODO visibility
        vis: item.visibility.into(),
        struct_token: Token![struct](span),
        ident: item.ident.into(),
        // TODO generics
        generics: Generics::default(),
        fields: syn::Fields::Named(fields),
        semi_token: None,
    }
}

fn fold_item_impl(item: WItemImpl) -> ItemImpl {
    let span = Span::call_site();

    let items = item
        .items
        .into_iter()
        .map(|impl_item| match impl_item {
            WImplItem::Type(impl_item) => ImplItem::Type(fold_impl_item_type(impl_item)),
            WImplItem::Fn(impl_item) => ImplItem::Fn(fold_impl_item_fn(impl_item)),
        })
        .collect();

    ItemImpl {
        attrs: Vec::new(),
        defaultness: None,
        unsafety: None,
        impl_token: Token![impl](span),
        // TODO generics
        generics: Generics::default(),
        trait_: item
            .trait_
            .map(|path| (None, path.into(), Token![for](span))),
        self_ty: Box::new(Type::Path(TypePath {
            qself: None,
            path: item.self_ty.into(),
        })),
        brace_token: Brace::default(),
        items,
    }
}

fn fold_impl_item_type(impl_item: WImplItemType) -> ImplItemType {
    let span = Span::call_site();

    ImplItemType {
        attrs: Vec::new(), // TODO visibility
        vis: syn::Visibility::Inherited,
        defaultness: None,
        type_token: Token![type](span),
        ident: impl_item.left_ident.into(),
        generics: Generics::default(),
        eq_token: Token![=](span),
        ty: Type::Path(TypePath {
            qself: None,
            path: impl_item.right_path.into(),
        }),
        semi_token: Token![;](span),
    }
}

fn fold_impl_item_fn(impl_item: WImplItemFn) -> ImplItemFn {
    let span = Span::call_site();

    let mut block = fold_block(impl_item.block);

    if let Some(result) = impl_item.result {
        block.stmts.push(Stmt::Expr(fold_right_expr(result), None));
    }

    ImplItemFn {
        // TODO: attrs
        attrs: Vec::new(),
        // TODO: visibility
        vis: syn::Visibility::Inherited,
        defaultness: None,
        sig: Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Token![fn](span),
            ident: impl_item.signature.ident.into(),
            generics: Generics::default(),
            paren_token: Paren::default(),
            inputs: Punctuated::from_iter(impl_item.signature.inputs.into_iter().map(|fn_arg| {
                if &fn_arg.ident.name == "self" {
                    // TODO: prefer typed self once it is well-supported as it is more regular
                    FnArg::Receiver(Receiver {
                        attrs: Vec::new(),
                        reference: match fn_arg.ty.reference {
                            WReference::Mutable | WReference::Immutable => {
                                Some((Token![&](span), None))
                            }
                            WReference::None => None,
                        },

                        mutability: match fn_arg.ty.reference {
                            WReference::Mutable => Some(Token![mut](span)),
                            WReference::Immutable | WReference::None => None,
                        },
                        self_token: Token![self](span),
                        colon_token: None,
                        ty: Box::new(fold_type(fn_arg.ty)),
                    })
                } else {
                    FnArg::Typed(syn::PatType {
                        attrs: Vec::new(),
                        pat: Box::new(Pat::Ident(syn::PatIdent {
                            attrs: Vec::new(),
                            by_ref: None,
                            mutability: None,
                            ident: fn_arg.ident.into(),
                            subpat: None,
                        })),
                        colon_token: Token![:](span),
                        ty: Box::new(fold_type(fn_arg.ty)),
                    })
                }
            })),
            variadic: None,
            output: syn::ReturnType::Type(
                Token![->](span),
                Box::new(fold_simple_type(impl_item.signature.output)),
            ),
        },
        block,
    }
}

fn fold_block(block: WBlock) -> Block {
    let mut stmts = Vec::new();

    for local in block.locals {
        let span = local.ident.span;

        let mut pat = Pat::Ident(PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident: local.ident.into(),
            subpat: None,
        });

        if let Some(ty) = local.ty {
            pat = Pat::Type(PatType {
                attrs: Vec::new(),
                pat: Box::new(pat),
                colon_token: Token![:](span),
                ty: Box::new(fold_type(ty)),
            });
        }

        stmts.push(syn::Stmt::Local(Local {
            attrs: vec![Attribute {
                pound_token: Token![#](span),
                style: syn::AttrStyle::Outer,
                bracket_token: Bracket::default(),
                meta: syn::Meta::NameValue(MetaNameValue {
                    path: path!(::mck::attr::tmp_original),
                    eq_token: Token![=](span),
                    value: create_expr_path(create_path_from_ident(local.original.into())),
                }),
            }],
            let_token: Token![let](span),
            pat,
            init: None,
            semi_token: Token![;](span),
        }));
    }

    for stmt in block.stmts {
        let span = Span::call_site();
        match stmt {
            WStmt::Assign(stmt) => {
                let right = fold_right_expr(stmt.right_expr);

                stmts.push(Stmt::Expr(
                    Expr::Assign(ExprAssign {
                        attrs: Vec::new(),
                        left: Box::new(create_expr_path(create_path_from_ident(
                            stmt.left_ident.into(),
                        ))),
                        eq_token: Token![=](span),
                        right: Box::new(right),
                    }),
                    Some(Token![;](span)),
                ));
            }
            WStmt::If(stmt) => {
                stmts.push(Stmt::Expr(
                    Expr::If(ExprIf {
                        attrs: Vec::new(),
                        if_token: Token![if](span),
                        cond: Box::new(fold_right_expr(stmt.condition)),
                        then_branch: fold_block(stmt.then_block),
                        else_branch: Some((
                            Token![else](span),
                            Box::new(Expr::Block(ExprBlock {
                                attrs: Vec::new(),
                                label: None,
                                block: fold_block(stmt.else_block),
                            })),
                        )),
                    }),
                    Some(Token![;](span)),
                ));
            }
        }
    }

    Block {
        brace_token: Brace::default(),
        stmts,
    }
}

fn fold_right_expr(right_expr: WExpr) -> Expr {
    let span = Span::call_site();
    match right_expr {
        WExpr::Move(ident) => create_expr_ident(ident.into()),
        WExpr::Call(expr) => {
            let args = Punctuated::from_iter(expr.args.into_iter().map(|arg| match arg {
                WCallArg::Ident(ident) => create_expr_ident(ident.into()),
                WCallArg::Literal(lit) => Expr::Lit(ExprLit {
                    attrs: Vec::new(),
                    lit,
                }),
            }));
            Expr::Call(ExprCall {
                attrs: Vec::new(),
                func: Box::new(create_expr_path(expr.fn_path.into())),
                paren_token: Paren::default(),
                // TODO args
                args,
            })
        }
        WExpr::Field(expr) => Expr::Field(ExprField {
            attrs: Vec::new(),
            base: Box::new(create_expr_ident(expr.base.into())),
            dot_token: Token![.](span),
            member: syn::Member::Named(expr.inner.into()),
        }),
        WExpr::Struct(expr) => {
            let fields = expr
                .fields
                .into_iter()
                .map(|(name, value)| FieldValue {
                    attrs: Vec::new(),
                    member: syn::Member::Named(name.into()),
                    colon_token: Some(Token![:](span)),
                    expr: create_expr_ident(value.into()),
                })
                .collect();

            Expr::Struct(ExprStruct {
                attrs: Vec::new(),
                qself: None,
                path: expr.type_path.into(),
                brace_token: Brace::default(),
                fields,
                dot2_token: None,
                rest: None,
            })
        }
        WExpr::Reference(expr) => {
            let inner = match expr {
                WExprReference::Ident(ident) => create_expr_ident(ident.into()),
                WExprReference::Field(expr) => Expr::Field(ExprField {
                    attrs: Vec::new(),
                    base: Box::new(create_expr_ident(expr.base.into())),
                    dot_token: Token![.](span),
                    member: syn::Member::Named(expr.inner.into()),
                }),
            };
            Expr::Reference(ExprReference {
                attrs: Vec::new(),
                and_token: Token![&](span),
                mutability: None,
                expr: Box::new(inner),
            })
        }
        WExpr::Lit(lit) => Expr::Lit(ExprLit {
            attrs: Vec::new(),
            lit,
        }),
    }
}

fn fold_type(ty: WType) -> Type {
    let span = Span::call_site();

    let simple_type = fold_simple_type(ty.inner);

    match ty.reference {
        WReference::Mutable => Type::Reference(TypeReference {
            and_token: Token![&](span),
            lifetime: None,
            mutability: Some(Token![mut](span)),
            elem: Box::new(simple_type),
        }),
        WReference::Immutable => Type::Reference(TypeReference {
            and_token: Token![&](span),
            lifetime: None,
            mutability: None,
            elem: Box::new(simple_type),
        }),
        WReference::None => simple_type,
    }
}

fn fold_simple_type(ty: WSimpleType) -> Type {
    let span = Span::call_site();
    match ty {
        WSimpleType::Bitvector(width) => create_mck_type("Bitvector", &[width], span),
        WSimpleType::Unsigned(width) => create_mck_type("Unsigned", &[width], span),
        WSimpleType::Signed(width) => create_mck_type("Signed", &[width], span),
        WSimpleType::BitvectorArray(array) => create_mck_type(
            "BitvectorArray",
            &[array.index_width, array.element_width],
            span,
        ),
        WSimpleType::Path(path) => Type::Path(TypePath {
            qself: None,
            path: path.into(),
        }),
        WSimpleType::Boolean => Type::Path(TypePath {
            qself: None,
            path: Path {
                leading_colon: Some(Token![::](span)),
                segments: Punctuated::from_iter(["mck", "concr", "Boolean"].into_iter().map(
                    |name| PathSegment {
                        ident: Ident::new(name, span),
                        arguments: PathArguments::None,
                    },
                )),
            },
        }),
    }
}

fn create_mck_type(name: &str, widths: &[u32], span: Span) -> Type {
    let width_arg = if !widths.is_empty() {
        let widths = Punctuated::from_iter(widths.iter().map(|width| {
            GenericArgument::Const(Expr::Lit(ExprLit {
                attrs: Vec::new(),
                lit: Lit::Int(LitInt::new(&width.to_string(), span)),
            }))
        }));

        syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: Token![<](span),
            args: widths,
            gt_token: Token![>](span),
        })
    } else {
        syn::PathArguments::None
    };

    let path = Path {
        leading_colon: Some(Token![::](span)),
        segments: Punctuated::from_iter([
            PathSegment {
                ident: Ident::new("machine_check", span),
                arguments: syn::PathArguments::None,
            },
            PathSegment {
                ident: Ident::new(name, span),
                arguments: width_arg,
            },
        ]),
    };
    Type::Path(TypePath { qself: None, path })
}

impl From<WVisibility> for Visibility {
    fn from(value: WVisibility) -> Self {
        match value {
            WVisibility::Public => Visibility::Public(Token![pub](Span::call_site())),
            WVisibility::Inherited => Visibility::Inherited,
        }
    }
}

impl From<WPath> for Path {
    fn from(path: WPath) -> Self {
        let span = Span::call_site();
        Path {
            leading_colon: if path.leading_colon {
                Some(Token![::](span))
            } else {
                None
            },

            segments: Punctuated::from_iter(path.segments.into_iter().map(|segment| {
                let arguments = match segment.generics {
                    Some(generics) => {
                        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            colon2_token: if generics.leading_colon {
                                Some(Token![::](span))
                            } else {
                                None
                            },
                            lt_token: Token![<](span),
                            args: Punctuated::from_iter(generics.inner.into_iter().map(
                                |generic| match generic {
                                    WGeneric::Type(ty) => {
                                        GenericArgument::Type(fold_simple_type(ty))
                                    }
                                    WGeneric::Const(value) => {
                                        GenericArgument::Const(Expr::Lit(ExprLit {
                                            attrs: Vec::new(),
                                            lit: Lit::Int(LitInt::new(&value.to_string(), span)),
                                        }))
                                    }
                                },
                            )),
                            gt_token: Token![>](span),
                        })
                    }
                    None => PathArguments::None,
                };
                PathSegment {
                    ident: segment.ident.into(),
                    arguments,
                }
            })),
        }
    }
}

impl From<WIdent> for Ident {
    fn from(ident: WIdent) -> Self {
        Ident::new(&ident.name, ident.span)
    }
}
