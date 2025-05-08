use core::panic;

use syn::{
    parse::Parser, punctuated::Punctuated, Block, Expr, ExprBlock, ExprPath, GenericArgument,
    Ident, ImplItem, ImplItemFn, Item, ItemImpl, ItemStruct, Pat, Path, PathArguments, Stmt, Token,
    Type, Visibility,
};

use crate::util::{extract_expr_ident, extract_expr_path, extract_path_ident};

use super::*;

impl WDescription<YSsa> {
    pub fn from_syn(item_iter: impl Iterator<Item = Item>) -> WDescription<YSsa> {
        let mut structs = Vec::new();
        let mut impls = Vec::new();
        for item in item_iter {
            match item {
                Item::Struct(item) => structs.push(fold_item_struct(item)),
                Item::Impl(item) => impls.push(fold_item_impl(item)),
                _ => panic!("Unexpected type of item: {:?}", item),
            }
        }

        WDescription { structs, impls }
    }
}

fn fold_item_struct(item: ItemStruct) -> WItemStruct {
    let mut derives = Vec::new();

    for attr in item.attrs {
        match attr.meta {
            syn::Meta::Path(_path) => todo!("path"),
            syn::Meta::List(meta) => {
                if meta.path.is_ident("derive") {
                    let meta_tokens = meta.tokens;
                    let parser = Punctuated::<Path, Token![,]>::parse_terminated;

                    let Ok(parsed) = parser.parse2(meta_tokens) else {
                        panic!("Cannot parse derive macro");
                    };
                    derives = parsed
                        .into_pairs()
                        .map(|pair| pair.into_value().into())
                        .collect();
                } else {
                    todo!("Non-derive meta list");
                }
            }
            syn::Meta::NameValue(meta) => {
                if meta.path.is_ident("allow") {
                    // TODO: copy allow
                } else if meta.path.is_ident("doc") {
                    // skip
                } else {
                    todo!("Name-value: {:?}", meta.path)
                }
            }
        }
    }

    let fields = match item.fields {
        syn::Fields::Named(fields_named) => fields_named
            .named
            .into_pairs()
            .map(|pair| {
                let field = pair.into_value();
                let Some(field_ident) = field.ident else {
                    panic!("Unexpected tuple struct");
                };
                WField {
                    ident: field_ident.into(),
                    ty: fold_simple_type(field.ty),
                }
            })
            .collect(),
        _ => panic!("Unexpected struct without named fields"),
    };

    WItemStruct {
        visibility: item.vis.into(),
        derives,
        ident: item.ident.into(),
        fields,
    }
}

fn fold_item_impl(item: ItemImpl) -> WItemImpl<YSsa> {
    let self_ty = {
        match *item.self_ty {
            Type::Path(ty) => {
                assert!(ty.qself.is_none());
                ty.path.into()
            }
            _ => panic!("Unexpected non-path type: {:?}", *item.self_ty),
        }
    };

    let trait_ = item.trait_.map(|(not, path, _for_token)| {
        assert!(not.is_none());
        path.into()
    });

    WItemImpl {
        self_ty,
        trait_,
        items: item.items.into_iter().map(fold_impl_item).collect(),
    }
}

fn fold_impl_item(impl_item: ImplItem) -> WImplItem<YSsa> {
    match impl_item {
        ImplItem::Fn(impl_item) => WImplItem::Fn(fold_impl_item_fn(impl_item)),
        ImplItem::Type(impl_item) => {
            let ty = impl_item.ty;
            let Type::Path(ty) = ty else {
                panic!("Unexpected non-path type: {:?}", ty);
            };
            WImplItem::Type(WImplItemType {
                left_ident: impl_item.ident.into(),
                right_path: ty.path.into(),
            })
        }
        _ => panic!("Unexpected type of impl item: {:?}", impl_item),
    }
}

fn fold_impl_item_fn(impl_item: ImplItemFn) -> WImplItemFn<YSsa> {
    let mut inputs = Vec::new();

    for input in impl_item.sig.inputs {
        // TODO: references
        match input {
            syn::FnArg::Receiver(receiver) => {
                let span = receiver.self_token.span;
                let reference = match receiver.reference {
                    Some(_) => {
                        if receiver.mutability.is_some() {
                            WReference::Mutable
                        } else {
                            WReference::Immutable
                        }
                    }
                    None => WReference::None,
                };

                inputs.push(WFnArg {
                    ident: WIdent {
                        name: String::from("self"),
                        span,
                    },
                    ty: WType {
                        reference,
                        inner: WSimpleType::Path(WPath {
                            leading_colon: false,
                            segments: vec![WPathSegment {
                                ident: WIdent {
                                    name: String::from("Self"),
                                    span,
                                },
                                generics: None,
                            }],
                        }),
                    },
                });
            }
            syn::FnArg::Typed(pat_type) => {
                let Pat::Ident(pat_ident) = *pat_type.pat else {
                    panic!("Unexpected non-ident pattern {:?}", pat_type);
                };

                inputs.push(WFnArg {
                    ident: pat_ident.ident.into(),
                    ty: fold_type(*pat_type.ty),
                });
            }
        }
    }

    let output = match impl_item.sig.output {
        syn::ReturnType::Default => panic!("Unexpected default function return type"),
        syn::ReturnType::Type(_rarrow, ty) => fold_simple_type(*ty),
    };

    let signature = WSignature {
        ident: impl_item.sig.ident.into(),
        inputs,
        output,
    };

    let mut locals = Vec::new();

    for stmt in &impl_item.block.stmts {
        if let Stmt::Local(local) = stmt {
            let mut original_ident = None;
            for attr in &local.attrs {
                match &attr.meta {
                    syn::Meta::Path(_path) => todo!("Local attr path"),
                    syn::Meta::List(_meta) => todo!("Local attr list"),
                    syn::Meta::NameValue(meta) => {
                        if meta.path.segments.len() == 3
                            && &meta.path.segments[0].ident.to_string() == "mck"
                            && &meta.path.segments[1].ident.to_string() == "attr"
                            && &meta.path.segments[2].ident.to_string() == "tmp_original"
                        {
                            let Expr::Path(ExprPath { ref path, .. }) = meta.value else {
                                panic!("Tmp original should contain a path");
                            };
                            original_ident = path.get_ident().cloned();
                        } else {
                            todo!("Local attr name-value: {:?}", meta)
                        }
                    }
                }
            }

            let mut pat = local.pat.clone();
            let mut ty = None;
            if let Pat::Type(pat_type) = pat {
                ty = Some(fold_type(*pat_type.ty));
                pat = *pat_type.pat;
            }

            let Pat::Ident(left_pat_ident) = pat else {
                panic!("Local pattern should be an ident: {:?}", pat)
            };

            locals.push(WLocal {
                ident: left_pat_ident.ident.into(),
                original: original_ident.unwrap().into(),
                ty: WPartialType(ty),
            });
        }
    }

    let (block, result) = fold_block(impl_item.block);

    WImplItemFn {
        signature,
        locals,
        block,
        result,
    }
}

fn fold_block(block: Block) -> (WBlock, Option<WExpr>) {
    let mut orig_stmts = block.stmts;
    let return_ident: Option<WExpr> = if let Some(Stmt::Expr(_, None)) = orig_stmts.last() {
        // has a return statement
        orig_stmts.pop().map(|stmt| {
            let Stmt::Expr(expr, None) = stmt else {
                panic!("Return statement should be an expression: {:?}", stmt);
            };
            fold_right_expr(expr)
        })
    } else {
        None
    };

    let mut stmts = Vec::new();

    for orig_stmt in orig_stmts {
        match orig_stmt {
            Stmt::Local(_) => {
                // do not process here
            }
            Stmt::Expr(expr, semi) => {
                assert!(semi.is_some());
                match expr {
                    syn::Expr::Assign(expr) => {
                        let Expr::Path(left_path) = *expr.left else {
                            panic!("Assignment left should be path");
                        };
                        let Some(left_ident) = left_path.path.get_ident() else {
                            panic!("Assignment left should be ident");
                        };

                        stmts.push(WStmt::Assign(WStmtAssign {
                            left_ident: left_ident.clone().into(),
                            right_expr: fold_right_expr(*expr.right),
                        }));
                    }
                    syn::Expr::If(expr_if) => {
                        let Expr::Block(ExprBlock {
                            block: else_block, ..
                        }) = *expr_if.else_branch.unwrap().1
                        else {
                            panic!("Else should have a block");
                        };

                        stmts.push(WStmt::If(WStmtIf {
                            condition: fold_right_expr(*expr_if.cond),
                            then_block: fold_block(expr_if.then_branch).0,
                            else_block: fold_block(else_block).0,
                        }));
                    }
                    syn::Expr::Block(expr_block) => {
                        // TODO: there should not be nested blocks here
                        let (mut block, result) = fold_block(expr_block.block);
                        assert!(result.is_none());
                        stmts.append(&mut block.stmts);
                    }
                    _ => panic!("Unexpected type of expression: {:?}", expr),
                };
            }
            _ => panic!("Unexpected type of statement: {:?}", orig_stmt),
        };
    }

    (WBlock { stmts }, return_ident)
}

fn fold_right_expr(expr: Expr) -> WExpr {
    match expr {
        Expr::Call(expr_call) => {
            let args = expr_call
                .args
                .iter()
                .map(|arg| match arg {
                    Expr::Lit(expr) => WCallArg::Literal(expr.lit.clone()),
                    Expr::Path(expr) => {
                        WCallArg::Ident(extract_path_ident(&expr.path).unwrap().clone().into())
                    }
                    _ => panic!(
                        "Unexpected non-literal and non-path call argument: {:?}",
                        arg
                    ),
                })
                .collect();

            WExpr::Call(WExprCall {
                fn_path: extract_expr_path(&expr_call.func).unwrap().clone().into(),
                args,
            })
        }
        Expr::Field(expr_field) => {
            let inner = match expr_field.member {
                syn::Member::Named(ident) => ident.into(),
                syn::Member::Unnamed(_index) => panic!("Unnamed members not supported"),
            };
            WExpr::Field(WExprField {
                base: extract_expr_ident(&expr_field.base).unwrap().clone().into(),
                inner,
            })
        }
        Expr::Path(expr_path) => {
            WExpr::Move(extract_path_ident(&expr_path.path).unwrap().clone().into())
        }
        Expr::Struct(expr_struct) => {
            let args = expr_struct
                .fields
                .into_pairs()
                .map(|pair| {
                    let field_value = pair.into_value();
                    let left = match field_value.member {
                        syn::Member::Named(ident) => ident.into(),
                        syn::Member::Unnamed(_) => panic!("Unnamed struct members not supported"),
                    };
                    let right = extract_expr_ident(&field_value.expr)
                        .unwrap()
                        .clone()
                        .into();
                    (left, right)
                })
                .collect();
            WExpr::Struct(WExprStruct {
                type_path: expr_struct.path.into(),
                fields: args,
            })
        }
        Expr::Reference(expr_reference) => match *expr_reference.expr {
            Expr::Path(expr_path) => {
                let Some(ident) = extract_path_ident(&expr_path.path) else {
                    panic!("Reference should be to an ident")
                };

                WExpr::Reference(WExprReference::Ident(ident.clone().into()))
            }
            Expr::Field(expr_field) => {
                let inner = match expr_field.member {
                    syn::Member::Named(ident) => ident.into(),
                    syn::Member::Unnamed(_index) => panic!("Unnamed members not supported"),
                };
                WExpr::Reference(WExprReference::Field(WExprField {
                    base: extract_expr_ident(&expr_field.base).unwrap().clone().into(),
                    inner,
                }))
            }
            _ => panic!(
                "Unexpected expression inside reference {:?}",
                expr_reference.expr
            ),
        },
        Expr::Lit(expr_lit) => WExpr::Lit(expr_lit.lit),
        _ => panic!("Unexpected right expression {:?}", expr),
    }
}

fn fold_type(mut ty: Type) -> WType {
    let reference = match ty {
        Type::Reference(type_reference) => {
            let mutable = type_reference.mutability.is_some();
            ty = *type_reference.elem;
            if mutable {
                WReference::Mutable
            } else {
                WReference::Immutable
            }
        }
        _ => WReference::None,
    };
    WType {
        reference,
        inner: fold_simple_type(ty),
    }
}

fn fold_simple_type(ty: Type) -> WSimpleType {
    match ty {
        Type::Path(ty) => {
            assert!(ty.qself.is_none());

            let mut known_type = None;
            if ty.path.leading_colon.is_some() {
                let mut segments_iter = ty.path.segments.clone().into_pairs();
                let first_segment = segments_iter.next().unwrap().into_value();

                if &first_segment.ident.to_string() == "machine_check"
                    && ty.path.segments.len() >= 2
                {
                    let second_segment = segments_iter.next().unwrap().into_value();
                    let arguments = second_segment.arguments;

                    if ty.path.segments.len() == 2 {
                        known_type = match second_segment.ident.to_string().as_str() {
                            "Bitvector" => Some(WSimpleType::Bitvector(
                                extract_generic_sizes(arguments, 1)[0],
                            )),
                            "Unsigned" => Some(WSimpleType::Unsigned(
                                extract_generic_sizes(arguments, 1)[0],
                            )),
                            "Signed" => {
                                Some(WSimpleType::Signed(extract_generic_sizes(arguments, 1)[0]))
                            }
                            "BitvectorArray" => {
                                let sizes = extract_generic_sizes(arguments, 2);
                                Some(WSimpleType::BitvectorArray(WTypeArray {
                                    index_width: sizes[0],
                                    element_width: sizes[1],
                                }))
                            }
                            _ => panic!("Unknown path type"),
                        };
                    } else if ty.path.segments.len() == 3 {
                        let third_segment = segments_iter.next().unwrap().into_value();
                        if second_segment.ident.to_string().as_str() == "internal"
                            && third_segment.ident.to_string().as_str() == "PanicResult"
                        {
                            known_type = Some(WSimpleType::PanicResult(None));
                        }
                    }
                } else if &first_segment.ident.to_string() == "mck" && ty.path.segments.len() == 3 {
                    let second_segment = segments_iter.next().unwrap().into_value();
                    let third_segment = segments_iter.next().unwrap().into_value();
                    if second_segment.ident.to_string().as_str() == "forward"
                        && third_segment.ident.to_string().as_str() == "PhiArg"
                    {
                        let mut inner_type = None;
                        if let PathArguments::AngleBracketed(generic_args) = third_segment.arguments
                        {
                            if let Some(GenericArgument::Type(inner)) = generic_args.args.first() {
                                inner_type = Some(Box::new(fold_simple_type(inner.clone())));
                            }
                        }

                        known_type = Some(WSimpleType::PhiArg(inner_type));
                    }
                }
            }
            if let Some(known_type) = known_type {
                known_type
            } else {
                //println!("Folded path type: {}", quote::quote!(#ty));
                WSimpleType::Path(ty.path.into())
            }
        }
        _ => panic!("Unexpected non-path type: {:?}", ty),
    }
}

fn extract_generic_sizes(arguments: PathArguments, expected_length: usize) -> Vec<u32> {
    let mut generic_sizes = Vec::new();
    match arguments {
        syn::PathArguments::None => {}
        syn::PathArguments::AngleBracketed(generic_args) => {
            assert_eq!(expected_length, generic_args.args.len());
            for arg in generic_args.args.into_iter() {
                match arg {
                    GenericArgument::Const(Expr::Lit(expr)) => match expr.lit {
                        syn::Lit::Int(lit_int) => {
                            let value: Result<u32, _> = lit_int.base10_parse();
                            let value = match value {
                                Ok(ok) => ok,
                                Err(err) => {
                                    panic!("Cannot parse generic argument: {:?}", err)
                                }
                            };
                            generic_sizes.push(value);
                        }
                        _ => panic!("Unexpected non-int generic argument"),
                    },
                    _ => panic!("Unexpected non-literal generic argument"),
                }
            }
        }
        syn::PathArguments::Parenthesized(_) => {
            panic!("Unexpected parenthesized generic arguments")
        }
    };
    generic_sizes
}

impl From<Visibility> for WVisibility {
    fn from(value: Visibility) -> Self {
        match value {
            syn::Visibility::Public(_) => WVisibility::Public,
            syn::Visibility::Restricted(_) => {
                panic!("Restricted visibility not supported")
            }
            syn::Visibility::Inherited => WVisibility::Inherited,
        }
    }
}

impl From<Path> for WPath {
    fn from(path: Path) -> Self {
        WPath {
            leading_colon: path.leading_colon.is_some(),
            segments: path
                .segments
                .into_iter()
                .map(|path_segment| {
                    let generics = match path_segment.arguments {
                        PathArguments::None => None,
                        PathArguments::AngleBracketed(generics) => Some(WGenerics {
                            leading_colon: generics.colon2_token.is_some(),
                            inner: generics
                                .args
                                .into_pairs()
                                .map(|pair| match pair.into_value() {
                                    GenericArgument::Type(ty) => {
                                        WGeneric::Type(fold_simple_type(ty))
                                    }
                                    GenericArgument::Const(expr) => {
                                        let Expr::Lit(expr) = expr else {
                                            panic!("Unexpected non-literal const generic argument");
                                        };
                                        let parsed: Result<u32, _> = match expr.lit {
                                            Lit::Int(lit_int) => lit_int.base10_parse(),
                                            _ => panic!(
                                                "Unexpected non-integer const generic argument"
                                            ),
                                        };
                                        let parsed = match parsed {
                                            Ok(ok) => ok,
                                            Err(err) => panic!(
                                                "Could not parse const generic argument: {}",
                                                err
                                            ),
                                        };
                                        WGeneric::Const(parsed)
                                    }
                                    _ => panic!("Unexpected type of generic argument"),
                                })
                                .collect(),
                        }),

                        PathArguments::Parenthesized(_generics) => {
                            panic!("Unexpected parenthesized generic arguments")
                        }
                    };

                    WPathSegment {
                        ident: path_segment.ident.into(),
                        generics,
                    }
                })
                .collect(),
        }
    }
}

impl From<Ident> for WIdent {
    fn from(ident: Ident) -> Self {
        WIdent {
            name: ident.to_string(),
            span: ident.span(),
        }
    }
}
